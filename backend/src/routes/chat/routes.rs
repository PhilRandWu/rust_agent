use crate::app::AppState;
use crate::routes::chat::handlers;
use axum::Router;
use axum::routing::post;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/chat", post(handlers::chat))
}
