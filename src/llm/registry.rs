use crate::config::model::{ModelConfig, ProviderConfig};
use crate::llm::provider::ModelProvider;

#[derive(Debug, Clone)]
pub struct ModelRegistry {
    config: ModelConfig,
}

impl ModelRegistry {
    pub fn new(config: ModelConfig) -> Self {
        Self { config }
    }

    pub fn main_provider(&self) -> ModelProvider {
        self.config.main_provider
    }

    pub fn vision_provider(&self) -> ModelProvider {
        self.config.vision_provider
    }

    pub fn main_config(&self) -> Option<&ProviderConfig> {
        self.provider_config(self.config.main_provider)
    }

    pub fn vision_config(&self) -> Option<&ProviderConfig> {
        self.provider_config(self.config.vision_provider)
    }

    pub fn provider_config(&self, provider: ModelProvider) -> Option<&ProviderConfig> {
        match provider {
            ModelProvider::Mock => None,
            ModelProvider::DeepSeek => Some(&self.config.deepseek),
            ModelProvider::Glm => Some(&self.config.glm),
            ModelProvider::Qwen => Some(&self.config.qwen),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn returns_none_for_mock_main_config() {
        let config = ModelConfig::from_values(&HashMap::new()).expect("config should load");
        let registry = ModelRegistry::new(config);

        assert_eq!(registry.main_provider(), ModelProvider::Mock);
        assert!(registry.main_config().is_none());
    }

    #[test]
    fn returns_selected_deepseek_main_config() {
        let values = HashMap::from([
            ("MAIN_MODEL_PROVIDER".to_string(), "deepseek".to_string()),
            ("DEEPSEEK_API_KEY".to_string(), "test-key".to_string()),
        ]);
        let config = ModelConfig::from_values(&values).expect("config should load");
        let registry = ModelRegistry::new(config);

        assert_eq!(registry.main_provider(), ModelProvider::DeepSeek);
        assert_eq!(registry.main_config().unwrap().model, "deepseek-chat");
    }
}
