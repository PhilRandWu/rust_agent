use axum::Router;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::config::app::AppConfig;
use crate::llm::registry::ModelRegistry;
use crate::llm::service::LlmService;
use crate::routes::{chat, health};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub model_registry: Arc<ModelRegistry>,
    pub llm_service: Arc<LlmService>,
}

impl AppState {
    pub fn new(config: AppConfig) -> anyhow::Result<Self> {
        let model_registry = ModelRegistry::new(config.model.clone());
        let llm_service = LlmService::from_registry(&model_registry)?;

        Ok(Self {
            config: Arc::new(config),
            model_registry: Arc::new(model_registry),
            llm_service: Arc::new(llm_service),
        })
    }
}

fn build_cors_layer(config: &AppConfig) -> anyhow::Result<CorsLayer> {
    let origins = config
        .cors
        .allowed_origins
        .iter()
        .map(|origin| origin.parse())
        .collect::<Result<Vec<_>, _>>()?;

    let methods = config
        .cors
        .allowed_methods
        .iter()
        .map(|method| method.parse())
        .collect::<Result<Vec<_>, _>>()?;

    let headers = config
        .cors
        .allowed_headers
        .iter()
        .map(|header| header.parse())
        .collect::<Result<Vec<_>, _>>()?;

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(methods)
        .allow_headers(headers)
        .max_age(config.cors.max_age);

    let cors = if config.cors.allow_credentials {
        cors.allow_credentials(true)
    } else {
        cors
    };

    Ok(cors)
}

pub fn build_router(state: AppState) -> anyhow::Result<Router> {
    let cors_layer = build_cors_layer(&state.config)?;

    Ok(Router::new()
        .merge(health::router())
        .merge(chat::router())
        .with_state(state)
        .layer(cors_layer))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Duration;

    use super::*;
    use crate::config::app::{CorsConfig, ServerConfig};
    use crate::config::model::ModelConfig;

    #[test]
    fn returns_error_for_invalid_cors_method() {
        let config = AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 7001,
            },
            cors: CorsConfig {
                allowed_origins: vec!["http://localhost:3000".to_string()],
                allowed_methods: vec!["INVALID METHOD".to_string()],
                allowed_headers: vec!["Content-Type".to_string()],
                allow_credentials: true,
                max_age: Duration::from_secs(3600),
            },
            model: ModelConfig::from_values(&HashMap::new()).expect("model config should load"),
        };

        let result = build_router(AppState::new(config).expect("state should build"));

        assert!(result.is_err());
    }
}
