use crate::agent::analysis::{AnalysisInput, AnalysisNode};
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
    let analysis_event = match AnalysisInput::from_chat_request(&request) {
        Ok(input) => match AnalysisNode::new(state.llm_service.main_client())
            .run(input)
            .await
        {
            Ok(output) => FrontendEvent::analysis_output(output),
            Err(error) => FrontendEvent::error(error.to_string()),
        },
        Err(error) => FrontendEvent::error(error.to_string()),
    };

    let events = vec![analysis_event, FrontendEvent::done()];

    let stream = tokio_stream::iter(events.into_iter().map(|event| {
        let data = event.to_sse_data().unwrap_or_else(|error| {
            FrontendEvent::error(error.to_string())
                .to_sse_data()
                .unwrap()
        });
        Ok(Event::default().data(data))
    }));
    Sse::new(stream)
}
