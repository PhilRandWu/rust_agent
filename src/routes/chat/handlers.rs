use crate::agent::adapters::registry::RouteRegistry;
use crate::agent::adapters::route::RouteFlow;
use crate::agent::event::AgentEvent;
use crate::agent::graph::{TraditionalGraph, collect_events};
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
    let agent_events = match RouteRegistry::new().resolve(&request) {
        Ok(route) => match route.flow {
            RouteFlow::Traditional => {
                let graph = TraditionalGraph::new(state.llm_service.main_client());
                collect_events(&graph, route.context).await
            }
        },
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
