use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelProvider {
    Mock,
    DeepSeek,
    Glm,
    Qwen,
}

impl FromStr for ModelProvider {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mock" => Ok(ModelProvider::Mock),
            "deepseek" => Ok(ModelProvider::DeepSeek),
            "glm" => Ok(ModelProvider::Glm),
            "qwen" => Ok(ModelProvider::Qwen),
            other => anyhow::bail!("Unknown model provider: {}", other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_model_provider_names() {
        assert_eq!(
            "mock".parse::<ModelProvider>().unwrap(),
            ModelProvider::Mock
        );
        assert_eq!(
            "deepseek".parse::<ModelProvider>().unwrap(),
            ModelProvider::DeepSeek
        );
        assert_eq!("glm".parse::<ModelProvider>().unwrap(), ModelProvider::Glm);
        assert_eq!(
            "qwen".parse::<ModelProvider>().unwrap(),
            ModelProvider::Qwen
        );
    }

    #[test]
    fn rejects_unknown_model_provider() {
        assert!("unknown".parse::<ModelProvider>().is_err());
    }
}
