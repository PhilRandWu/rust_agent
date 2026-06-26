use std::sync::Arc;
use axum::Router;
use crate::config::app::AppConfig;
use crate::routes::health;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(health::router())
        .with_state(state)
}
