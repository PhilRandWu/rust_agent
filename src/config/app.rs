use crate::config::model::ModelConfig;
use std::{collections::HashMap, env, time::Duration};

pub struct AppConfig {
    pub server: ServerConfig,
    pub cors: CorsConfig,
    pub model: ModelConfig,
}

pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub allow_credentials: bool,
    pub max_age: Duration,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let values = env::vars().collect();
        Self::from_values(&values)
    }

    pub fn from_values(values: &HashMap<String, String>) -> anyhow::Result<Self> {
        let host = values
            .get("APP_HOST")
            .cloned()
            .unwrap_or_else(|| "127.0.0.1".to_string());

        let port = values
            .get("APP_PORT")
            .map(String::as_str)
            .unwrap_or("7001")
            .parse::<u16>()?;

        let allowed_origins =
            value_list(values, "CORS_ALLOWED_ORIGINS", &["http://localhost:3000"]);

        let allowed_methods = value_list(
            values,
            "CORS_ALLOWED_METHODS",
            &["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"],
        );

        let allowed_headers = value_list(
            values,
            "CORS_ALLOWED_HEADERS",
            &["Content-Type", "Authorization", "Accept"],
        );

        let allow_credentials = values
            .get("CORS_ALLOW_CREDENTIALS")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(true);

        let max_age = values
            .get("CORS_MAX_AGE")
            .map(|s| s.parse::<u64>().unwrap_or(3600))
            .map(Duration::from_secs)
            .unwrap_or_else(|| Duration::from_secs(3600));

        let model = ModelConfig::from_values(values)?;

        Ok(Self {
            server: ServerConfig { host, port },
            cors: CorsConfig {
                allowed_origins,
                allowed_methods,
                allowed_headers,
                allow_credentials,
                max_age,
            },
            model,
        })
    }
}

fn value_list(values: &HashMap<String, String>, key: &str, default: &[&str]) -> Vec<String> {
    values
        .get(key)
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_else(|| default.iter().map(ToString::to_string).collect())
}

impl ServerConfig {
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_server_config_from_values() {
        let values = HashMap::from([
            ("APP_HOST".to_string(), "127.0.0.1".to_string()),
            ("APP_PORT".to_string(), "7001".to_string()),
        ]);

        let config = AppConfig::from_values(&values).expect("config should load");

        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 7001);
    }

    #[test]
    fn ignores_empty_items_when_parsing_lists() {
        let values = HashMap::from([(
            "CORS_ALLOWED_ORIGINS".to_string(),
            "http://localhost:3000, ,http://localhost:5173".to_string(),
        )]);

        let config = AppConfig::from_values(&values).expect("config should load");

        assert_eq!(
            config.cors.allowed_origins,
            vec!["http://localhost:3000", "http://localhost:5173"]
        );
    }
}
