use crate::llm::provider::ModelProvider;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub main_provider: ModelProvider,
    pub vision_provider: ModelProvider,
    pub deepseek: ProviderConfig,
    pub glm: ProviderConfig,
    pub qwen: ProviderConfig,
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub base_url: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl ProviderConfig {
    pub fn from_values(
        values: &HashMap<String, String>,
        prefix: &str,
        default_base_url: &str,
        default_model: &str,
        default_temperature: f32,
        default_max_tokens: u32,
    ) -> anyhow::Result<Self> {
        let api_key = optional_value(values, &format!("{prefix}_API_KEY"));

        let base_url = values
            .get(&format!("{prefix}_BASE_URL"))
            .cloned()
            .unwrap_or_else(|| default_base_url.to_string());

        let model = values
            .get(&format!("{prefix}_MODEL"))
            .cloned()
            .unwrap_or_else(|| default_model.to_string());

        let temperature = values
            .get(&format!("{prefix}_TEMPERATURE"))
            .map(String::as_str)
            .unwrap_or("")
            .parse::<f32>()
            .unwrap_or(default_temperature);

        let max_tokens = values
            .get(&format!("{prefix}_MAX_TOKENS"))
            .map(String::as_str)
            .unwrap_or("")
            .parse::<u32>()
            .unwrap_or(default_max_tokens);

        Ok(Self {
            api_key,
            base_url,
            model,
            temperature,
            max_tokens,
        })
    }

    fn require_api_key(&self, name: &str) -> anyhow::Result<()> {
        if self.api_key.as_deref().unwrap_or("").trim().is_empty() {
            anyhow::bail!("{name} is required for selected model provider");
        }
        Ok(())
    }
}

fn optional_value(values: &HashMap<String, String>, key: &str) -> Option<String> {
    values
        .get(key)
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .map(ToString::to_string)
}

impl ModelConfig {
    pub fn from_values(values: &HashMap<String, String>) -> anyhow::Result<Self> {
        let main_provider = values
            .get("MAIN_MODEL_PROVIDER")
            .map(String::as_str)
            .unwrap_or("mock")
            .parse::<ModelProvider>()?;

        let vision_provider = values
            .get("VISION_MODEL_PROVIDER")
            .map(String::as_str)
            .unwrap_or("mock")
            .parse::<ModelProvider>()?;

        let config = Self {
            main_provider,
            vision_provider,
            deepseek: ProviderConfig::from_values(
                values,
                "DEEPSEEK",
                "https://api.deepseek.com",
                "deepseek-chat",
                0.0,
                8192,
            )?,
            glm: ProviderConfig::from_values(
                values,
                "GLM",
                "https://open.bigmodel.cn/api/paas/v4/",
                "glm-4-flash",
                0.0,
                8192,
            )?,
            qwen: ProviderConfig::from_values(
                values,
                "QWEN",
                "https://dashscope.aliyuncs.com/compatible-mode/v1",
                "qwen-vl-max",
                0.1,
                32768,
            )?,
        };
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> anyhow::Result<()> {
        match self.main_provider {
            ModelProvider::Mock => {}
            ModelProvider::DeepSeek => self.deepseek.require_api_key("DEEPSEEK_API_KEY")?,
            ModelProvider::Glm => self.glm.require_api_key("GLM_API_KEY")?,
            ModelProvider::Qwen => anyhow::bail!("qwen cannot be used as MAIN_MODEL_PROVIDER"),
        }

        match self.vision_provider {
            ModelProvider::Mock => {}
            ModelProvider::Qwen => self.qwen.require_api_key("QWEN_API_KEY")?,
            ModelProvider::DeepSeek | ModelProvider::Glm => {
                anyhow::bail!("vision provider must be mock or qwen")
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_mock_providers() {
        let values = HashMap::new();

        let config = ModelConfig::from_values(&values).expect("config should load");

        assert_eq!(config.main_provider, ModelProvider::Mock);
        assert_eq!(config.vision_provider, ModelProvider::Mock);
        assert!(config.deepseek.api_key.is_none());
    }

    #[test]
    fn reads_vision_provider_from_vision_variable() {
        let values = HashMap::from([("VISION_MODEL_PROVIDER".to_string(), "qwen".to_string())]);

        let config = ModelConfig::from_values(&values).expect_err("qwen needs an api key");

        assert!(config.to_string().contains("QWEN_API_KEY"));
    }

    #[test]
    fn rejects_deepseek_without_api_key() {
        let values = HashMap::from([("MAIN_MODEL_PROVIDER".to_string(), "deepseek".to_string())]);

        let error = ModelConfig::from_values(&values).expect_err("deepseek needs an api key");

        assert!(error.to_string().contains("DEEPSEEK_API_KEY"));
    }

    #[test]
    fn accepts_deepseek_with_api_key() {
        let values = HashMap::from([
            ("MAIN_MODEL_PROVIDER".to_string(), "deepseek".to_string()),
            ("DEEPSEEK_API_KEY".to_string(), "test-key".to_string()),
        ]);

        let config = ModelConfig::from_values(&values).expect("config should load");

        assert_eq!(config.main_provider, ModelProvider::DeepSeek);
        assert_eq!(config.deepseek.api_key.as_deref(), Some("test-key"));
    }

    #[test]
    fn rejects_qwen_as_main_provider() {
        let values = HashMap::from([("MAIN_MODEL_PROVIDER".to_string(), "qwen".to_string())]);

        let error = ModelConfig::from_values(&values).expect_err("qwen cannot be main provider");

        assert!(error.to_string().contains("MAIN_MODEL_PROVIDER"));
    }

    #[test]
    fn rejects_glm_as_vision_provider() {
        let values = HashMap::from([("VISION_MODEL_PROVIDER".to_string(), "glm".to_string())]);

        let error = ModelConfig::from_values(&values).expect_err("glm cannot be vision provider");

        assert!(error.to_string().contains("vision provider"));
    }
}
