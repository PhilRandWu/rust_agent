use crate::agent::analysis::{AnalysisInput, AnalysisNode};
use crate::agent::context::AgentContext;
use crate::agent::event::AgentEvent;
use crate::agent::runner::AgentRunner;
use crate::app::AppState;
use crate::routes::chat::dto::ChatRequest;
use crate::sse::event::FrontendEvent;
use axum::{
    extract::{Json, State},
    response::sse::{Event, Sse},
};
use std::convert::Infallible;
use tokio_stream::Stream;

pub async fn chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let agent_events = match AgentContext::from_chat_request(&request) {
        Ok(context) => {
            AgentRunner::new(state.llm_service.main_client())
                .run(context)
                .await
        }
        Err(error) => vec![AgentEvent::Error(error.to_string()), AgentEvent::Done],
    };

    let stream = tokio_stream::iter(agent_events.into_iter().map(|event| {
        let frontend_event = FrontendEvent::from_agent_event(event);

        let data = frontend_event.to_sse_data().unwrap_or_else(|error| {
            FrontendEvent::error(error.to_string())
                .to_sse_data()
                .unwrap()
        });
        Ok(Event::default().data(data))
    }));
    Sse::new(stream)
}
