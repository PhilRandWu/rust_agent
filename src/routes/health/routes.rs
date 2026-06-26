use axum::Router;
use axum::routing::get;
use crate::app::AppState;
use crate::routes::health::handlers;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health))
}
