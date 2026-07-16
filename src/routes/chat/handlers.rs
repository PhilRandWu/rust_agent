use crate::agent::adapters::registry::RouteRegistry;
use crate::agent::adapters::route::RouteFlow;
use crate::agent::event::AgentEvent;
use crate::agent::graph::{AgentGraph, TraditionalGraph};
use crate::app::AppState;
use crate::llm::provider::ModelProvider;
use crate::routes::chat::dto::ChatRequest;
use crate::sse::event::FrontendEvent;
use async_stream::stream;
use axum::Json;
use axum::extract::State;
use axum::response::Sse;
use axum::response::sse::{Event, KeepAlive};
use futures::Stream;
use std::convert::Infallible;
use std::time::Duration;
use tokio::sync::mpsc;

const AGENT_EVENT_CHANNEL_SIZE: usize = 32;

#[tracing::instrument(
    skip(state, request),
    fields(
        project_id = request.project_id.as_deref().unwrap_or("-"),
        msg_count = request.messages.len(),
        has_mock_config = request.mock_config.is_some(),
    )
)]
pub async fn chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = mpsc::channel::<AgentEvent>(AGENT_EVENT_CHANNEL_SIZE);
    let mut early_error: Option<String> = None;

    match RouteRegistry::new().resolve(&request) {
        Ok(mut route) => {
            let main_is_mock = matches!(state.config.model.main_provider, ModelProvider::Mock);
            route.context.ensure_dev_mock_defaults(main_is_mock);
            let main_client = state.llm_service.main_client();
            match route.flow {
                RouteFlow::Traditional => {
                    let ctx = route.context;
                    let gate = state.llm_gate.clone();
                    let session_store = state.session_store.clone();
                    tokio::spawn(async move {
                        let graph = TraditionalGraph::with_gate_and_session(
                            main_client,
                            gate,
                            session_store,
                        );
                        graph.run_streaming(ctx, tx).await;
                    });
                }
            }
        }
        Err(error) => {
            early_error = Some(error.to_string());
            drop(tx);
        }
    }

    let sse_stream = stream! {
        if let Some(msg) = early_error {
            yield agent_event_to_sse(AgentEvent::Error(msg));
            yield agent_event_to_sse(AgentEvent::Done);
            return;
        }
        let mut rx = rx;
        while let Some(evt) = rx.recv().await {
            yield agent_event_to_sse(evt);
        }
    };
    Sse::new(sse_stream).keep_alive(default_keep_alive())
}

fn default_keep_alive() -> KeepAlive {
    KeepAlive::new()
        .interval(Duration::from_secs(15))
        .text("keep-alive")
}

fn agent_event_to_sse(event: AgentEvent) -> Result<Event, Infallible> {
    let frontend_event = FrontendEvent::from_agent_event(event);
    let data = frontend_event.to_sse_data().unwrap_or_else(|error| {
        FrontendEvent::error(error.to_string())
            .to_sse_data()
            .unwrap()
    });
    Ok(Event::default().data(data))
}

#[cfg(test)]
mod tests {
    use crate::app::{AppState, build_router};
    use crate::config::app::AppConfig;
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode, header};
    use http_body_util::BodyExt;
    use serde_json::json;
    use std::collections::HashMap;
    use tower::ServiceExt;

    fn build_test_router() -> axum::Router {
        let config = AppConfig::from_values(&HashMap::new()).expect("mock app config should load");
        let state = AppState::new(config).expect("app state should build");
        build_router(state).expect("router should build")
    }

    async fn collect_body_utf8(response: axum::response::Response) -> String {
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body should collect")
            .to_bytes();
        String::from_utf8(bytes.to_vec()).expect("body should be utf-8")
    }

    fn all_nodes_mock_config() -> serde_json::Value {
        json!({
            "analysisNode": true,
            "intentNode": true,
            "capabilityNode": true,
            "uiNode": true,
            "componentNode": true,
            "structureNode": true,
            "dependencyNode": true,
            "typeNode": true,
            "utilsNode": true,
            "mockDataNode": true,
            "serviceNode": true,
            "hooksNode": true,
            "componentGenNode": true,
            "pageGenNode": true,
            "layoutNode": true,
            "styleGenNode": true,
            "appGenNode": true
        })
    }

    #[tokio::test]
    async fn sse_streams_full_pipeline_over_http() {
        let router = build_test_router();
        let payload = json!({
            "messages": [
                { "role": "user", "content": "build a todo app" }
            ],
            "mockConfig": all_nodes_mock_config()
        });

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/chat")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_string();
        assert!(
            content_type.starts_with("text/event-stream"),
            "expected SSE content-type, got: {content_type}"
        );

        let body = collect_body_utf8(response).await;

        let data_lines: Vec<&str> = body
            .lines()
            .filter_map(|line| line.strip_prefix("data: "))
            .collect();

        assert!(
            data_lines.len() >= 5,
            "expected multiple SSE data frames, got {}: {body}",
            data_lines.len()
        );

        assert!(
            data_lines
                .first()
                .map(|line| line.contains(r#""type":"analysis""#))
                .unwrap_or(false),
            "first frame should be analysis, got: {body}"
        );
        assert!(
            data_lines
                .last()
                .map(|line| line.contains(r#""type":"done""#))
                .unwrap_or(false),
            "last frame should be done, got: {body}"
        );
        assert!(
            data_lines
                .iter()
                .any(|line| line.contains(r#""type":"files""#)),
            "expected a files frame, got: {body}"
        );
    }

    #[tokio::test]
    async fn sse_returns_error_when_no_user_message() {
        let router = build_test_router();
        let payload = json!({ "messages": [] });

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/chat")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");

        assert_eq!(response.status(), StatusCode::OK);
        let body = collect_body_utf8(response).await;

        let data_lines: Vec<&str> = body
            .lines()
            .filter_map(|line| line.strip_prefix("data: "))
            .collect();

        assert_eq!(
            data_lines.len(),
            2,
            "expected exactly 2 frames (error + done), got: {body}"
        );
        assert!(data_lines[0].contains(r#""type":"error""#));
        assert!(data_lines[1].contains(r#""type":"done""#));
    }

    #[tokio::test]
    async fn sse_emits_two_files_frames_from_assemble_and_post_process() {
        let router = build_test_router();
        let payload = json!({
            "messages": [
                { "role": "user", "content": "build a todo app" }
            ],
            "mockConfig": all_nodes_mock_config()
        });

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/chat")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");

        assert_eq!(response.status(), StatusCode::OK);
        let body = collect_body_utf8(response).await;

        let files_count = body
            .lines()
            .filter(|line| line.contains(r#""type":"files""#))
            .count();

        assert_eq!(
            files_count, 2,
            "expected exactly 2 files frames (assemble + postProcess), got {files_count}: {body}"
        );
    }

    #[tokio::test]
    async fn sse_uses_dev_mock_fallback_when_no_mock_config() {
        let router = build_test_router();
        let payload = json!({
            "messages": [
                { "role": "user", "content": "build a todo app" }
            ]
        });

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/chat")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");

        assert_eq!(response.status(), StatusCode::OK);
        let body = collect_body_utf8(response).await;

        assert!(
            !body.contains(r#""type":"error""#),
            "dev mock fallback should not emit error frame, got: {body}"
        );
        assert!(
            body.contains(r#""type":"files""#),
            "dev mock fallback should stream files frames, got: {body}"
        );
        assert!(
            body.contains(r#""type":"done""#),
            "stream should end with done, got: {body}"
        );
    }

    #[tokio::test]
    async fn sse_emits_session_frame_and_commits_version_when_project_id_present() {
        let router = build_test_router();
        let payload = json!({
            "messages": [
                { "role": "user", "content": "build a todo app" }
            ],
            "projectId": "proj-session-1",
            "mockConfig": all_nodes_mock_config()
        });

        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/chat")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .expect("request should build"),
            )
            .await
            .expect("router should respond");

        assert_eq!(response.status(), StatusCode::OK);
        let body = collect_body_utf8(response).await;

        assert!(
            body.contains(r#""type":"session""#),
            "expected a session frame after postProcess, got: {body}"
        );
        assert!(
            body.contains(r#""version":1"#),
            "expected version=1 for first commit, got: {body}"
        );
        assert!(
            body.contains(r#""operation":"create""#),
            "first commit should be operation=create"
        );

        let session_idx = body.find(r#""type":"session""#).unwrap();
        let done_idx = body.find(r#""type":"done""#).unwrap();
        let last_files_idx = body.rfind(r#""type":"files""#).unwrap();
        assert!(
            last_files_idx < session_idx,
            "session should come after last files frame"
        );
        assert!(session_idx < done_idx, "session should precede done");

        let list_resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/session/proj-session-1/versions")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(list_resp.status(), StatusCode::OK);
        let list_json: serde_json::Value =
            serde_json::from_slice(&list_resp.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(list_json["versions"][0]["version"], 1);
        assert_eq!(list_json["versions"][0]["operation"], "create");
    }

    #[tokio::test]
    async fn sse_does_not_commit_session_when_project_id_absent() {
        let router = build_test_router();
        let payload = json!({
            "messages": [{ "role": "user", "content": "todo" }],
            "mockConfig": all_nodes_mock_config()
        });

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/chat")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .expect("req"),
            )
            .await
            .expect("resp");
        let body = collect_body_utf8(response).await;
        assert!(
            !body.contains(r#""type":"session""#),
            "no session frame expected when projectId missing: {body}"
        );
    }
}
