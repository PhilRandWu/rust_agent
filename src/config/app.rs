use std::{collections::HashMap, env};

pub struct AppConfig {
    pub server: ServerConfig,
}

pub struct ServerConfig {
    pub host: String,
    pub port: u16,
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

        Ok(Self {
            server: ServerConfig { host, port },
        })
    }
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
}
