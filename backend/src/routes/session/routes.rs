use crate::app::AppState;
use crate::routes::session::handlers;
use axum::Router;
use axum::routing::{delete, get};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/session/{project_id}/versions",
            get(handlers::list_versions),
        )
        .route(
            "/api/session/{project_id}/versions/{version}",
            get(handlers::get_version),
        )
        .route(
            "/api/session/{project_id}",
            delete(handlers::delete_session),
        )
}
