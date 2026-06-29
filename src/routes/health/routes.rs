use crate::app::AppState;
use crate::routes::health::handlers;
use axum::Router;
use axum::routing::get;

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(handlers::health))
}
