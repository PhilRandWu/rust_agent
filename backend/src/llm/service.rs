use crate::llm::client::LlmClient;
use crate::llm::mock::MockLlmClient;
use crate::llm::openai_compatible::OpenAiCompatibleClient;
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
            main_client: build_client(registry.main_provider(), registry.main_config())?,
            vision_client: build_client(registry.vision_provider(), registry.vision_config())?,
        })
    }

    pub fn main_client(&self) -> Arc<dyn LlmClient> {
        Arc::clone(&self.main_client)
    }

    pub fn vision_client(&self) -> Arc<dyn LlmClient> {
        Arc::clone(&self.vision_client)
    }
}

fn build_client(
    provider: ModelProvider,
    config: Option<&crate::config::model::ProviderConfig>,
) -> anyhow::Result<Arc<dyn LlmClient>> {
    match provider {
        ModelProvider::Mock => Ok(Arc::new(MockLlmClient::default())),
        ModelProvider::DeepSeek | ModelProvider::Glm | ModelProvider::Qwen => {
            let config = config
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("missing provider config for {:?}", provider))?;
            Ok(Arc::new(OpenAiCompatibleClient::new(provider, config)?))
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

        assert!(serde_json::from_str::<serde_json::Value>(&response.content).is_ok());
    }

    #[test]
    fn creates_real_provider_client() {
        let values = HashMap::from([
            ("MAIN_MODEL_PROVIDER".to_string(), "deepseek".to_string()),
            ("DEEPSEEK_API_KEY".to_string(), "test-key".to_string()),
        ]);
        let config = ModelConfig::from_values(&values).expect("config should load");
        let registry = ModelRegistry::new(config);

        let service = LlmService::from_registry(&registry);

        assert!(service.is_ok());
    }
}
