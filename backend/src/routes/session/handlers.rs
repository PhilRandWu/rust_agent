use crate::app::AppState;
use crate::routes::session::dto::{GetVersionResponse, ListVersionsResponse};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

pub async fn list_versions(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    let versions = state.session_store.list_versions(&project_id).await;
    Json(ListVersionsResponse {
        project_id,
        versions,
    })
}

pub async fn get_version(
    State(state): State<AppState>,
    Path((project_id, version)): Path<(String, u32)>,
) -> impl IntoResponse {
    match state.session_store.get_version(&project_id, version).await {
        Some(snapshot) => (
            StatusCode::OK,
            Json(GetVersionResponse {
                project_id,
                snapshot,
            }),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "version_not_found",
                "projectId": project_id,
                "version": version,
            })),
        )
            .into_response(),
    }
}

pub async fn delete_session(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    state.session_store.delete_session(&project_id).await;
    StatusCode::NO_CONTENT
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

    fn build_test_router() -> (axum::Router, AppState) {
        let config = AppConfig::from_values(&HashMap::new()).expect("cfg");
        let state = AppState::new(config).expect("state");
        let router = build_router(state.clone()).expect("router");
        (router, state)
    }

    async fn body_to_json(resp: axum::response::Response) -> serde_json::Value {
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn list_versions_empty_returns_empty_array() {
        let (router, _) = build_test_router();
        let resp = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/session/unknown/versions")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_to_json(resp).await;
        assert_eq!(json["projectId"], "unknown");
        assert_eq!(json["versions"], json!([]));
    }

    #[tokio::test]
    async fn get_missing_version_returns_404() {
        let (router, _) = build_test_router();
        let resp = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/session/nope/versions/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_session_returns_204() {
        let (router, _) = build_test_router();
        let resp = router
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri("/api/session/whatever")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn full_lifecycle_via_direct_store() {
        let (router, state) = build_test_router();
        use crate::session::{Operation, VersionSnapshot};
        use std::collections::BTreeMap;
        let snap = VersionSnapshot {
            version: 0,
            timestamp_ms: 100,
            operation: Operation::Create,
            prompt: "generate todo".into(),
            file_count: 1,
            files: BTreeMap::from([("/App.tsx".into(), "export default () => null;".into())]),
        };
        let assigned = state.session_store.append_version("proj-1", snap).await;
        assert_eq!(assigned, 1);

        // GET list
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/session/proj-1/versions")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_to_json(resp).await;
        assert_eq!(json["versions"][0]["version"], 1);
        assert_eq!(json["versions"][0]["operation"], "create");
        assert!(
            !json["versions"][0]
                .as_object()
                .unwrap()
                .contains_key("files")
        );

        // GET specific version → full snapshot
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/session/proj-1/versions/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_to_json(resp).await;
        assert_eq!(json["version"], 1);
        assert_eq!(json["files"]["/App.tsx"], "export default () => null;");

        // DELETE
        let resp = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri("/api/session/proj-1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        assert_eq!(state.session_store.latest_version("proj-1").await, 0);

        // Explicitly avoid unused import warning in tests
        let _ = header::CONTENT_TYPE;
    }
}
