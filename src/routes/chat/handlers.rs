use crate::app::AppState;
use crate::routes::chat::converter::chat_request_to_llm_messages;
use crate::routes::chat::dto::ChatRequest;
use crate::sse::event::FrontendEvent;
use axum::extract::State;
use axum::{
    extract::Json,
    response::sse::{Event, Sse},
};
use std::convert::Infallible;
use tokio_stream::Stream;

pub async fn chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let llm_messages = chat_request_to_llm_messages(&request);

    let analysis_event = match state.llm_service.main_client().chat(&llm_messages).await {
        Ok(response) => FrontendEvent::analysis_text(response.content),
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
