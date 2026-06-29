use crate::routes::chat::dto::ChatRequest;
use crate::sse::event::FrontendEvent;
use axum::{
    extract::Json,
    response::sse::{Event, Sse},
};
use std::convert::Infallible;
use tokio_stream::Stream;

pub async fn chat(
    Json(_request): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let events = vec![FrontendEvent::placeholder_analysis(), FrontendEvent::done()];

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
