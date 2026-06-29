use crate::llm::client::LlmClient;
use crate::llm::mock::MockLlmClient;
use crate::llm::provider::ModelProvider;
use crate::llm::registry::ModelRegistry;
use std::sync::Arc;

pub struct LlmService {
    main_client: Arc<dyn LlmClient>,
    vision_client: Arc<dyn LlmClient>,
}

impl LlmService {
    pub fn from_registry(registry: &ModelRegistry) -> anyhow::Result<Self> {
        Ok(Self {
            main_client: build_client(registry.main_provider())?,
            vision_client: build_client(registry.vision_provider())?,
        })
    }

    pub fn main_client(&self) -> Arc<dyn LlmClient> {
        Arc::clone(&self.main_client)
    }

    pub fn vision_client(&self) -> Arc<dyn LlmClient> {
        Arc::clone(&self.vision_client)
    }
}

fn build_client(provider: ModelProvider) -> anyhow::Result<Arc<dyn LlmClient>> {
    match provider {
        ModelProvider::Mock => Ok(Arc::new(MockLlmClient::default())),
        ModelProvider::DeepSeek | ModelProvider::Glm | ModelProvider::Qwen => {
            anyhow::bail!(
                "real model provider is not supported before async-openai client is implemented"
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::config::model::ModelConfig;
    use crate::llm::message::LlmMessage;

    #[tokio::test]
    async fn creates_mock_main_client() {
        let config = ModelConfig::from_values(&HashMap::new()).expect("config should load");
        let registry = ModelRegistry::new(config);
        let service = LlmService::from_registry(&registry).expect("service should load");

        let response = service
            .main_client()
            .chat(&[LlmMessage::user("hello")])
            .await
            .unwrap();

        assert_eq!(response.content, "Mock LLM Response");
    }

    #[test]
    fn rejects_real_provider_until_async_openai_client_exists() {
        let values = HashMap::from([
            ("MAIN_MODEL_PROVIDER".to_string(), "deepseek".to_string()),
            ("DEEPSEEK_API_KEY".to_string(), "test-key".to_string()),
        ]);
        let config = ModelConfig::from_values(&values).expect("config should load");
        let registry = ModelRegistry::new(config);

        let error = match LlmService::from_registry(&registry) {
            Ok(_) => panic!("real provider should be unsupported"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("async-openai"));
    }
}
