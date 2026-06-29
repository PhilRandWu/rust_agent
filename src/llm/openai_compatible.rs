use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs,
};

use crate::config::model::ProviderConfig;
use crate::llm::client::LlmClient;
use crate::llm::message::{LlmMessage, LlmResponse, LlmRole};
use crate::llm::provider::ModelProvider;

pub struct OpenAiCompatibleClient {
    provider: ModelProvider,
    config: ProviderConfig,
    client: Client<OpenAIConfig>,
}

impl OpenAiCompatibleClient {
    pub fn new(provider: ModelProvider, config: ProviderConfig) -> anyhow::Result<Self> {
        let api_key = config
            .api_key
            .clone()
            .filter(|key| !key.trim().is_empty())
            .ok_or_else(|| anyhow::anyhow!("api key is required for {:?}", provider))?;

        let openai_config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(config.base_url.clone());

        Ok(Self {
            provider,
            config,
            client: Client::with_config(openai_config),
        })
    }

    pub fn provider(&self) -> ModelProvider {
        self.provider
    }
}

#[async_trait::async_trait]
impl LlmClient for OpenAiCompatibleClient {
    async fn chat(&self, messages: &[LlmMessage]) -> anyhow::Result<LlmResponse> {
        let openai_messages = messages
            .iter()
            .map(to_openai_message)
            .collect::<anyhow::Result<Vec<_>>>()?;

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.config.model)
            .messages(openai_messages)
            .temperature(self.config.temperature)
            .max_tokens(self.config.max_tokens)
            .build()?;

        let response = self.client.chat().create(request).await?;
        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or_else(|| {
                anyhow::anyhow!("empty chat completion response from {:?}", self.provider)
            })?;

        Ok(LlmResponse::new(content))
    }
}

pub(crate) fn to_openai_message(
    message: &LlmMessage,
) -> anyhow::Result<ChatCompletionRequestMessage> {
    match message.role {
        LlmRole::System => Ok(ChatCompletionRequestSystemMessageArgs::default()
            .content(message.content.clone())
            .build()?
            .into()),
        LlmRole::User => Ok(ChatCompletionRequestUserMessageArgs::default()
            .content(message.content.clone())
            .build()?
            .into()),
        LlmRole::Assistant => Ok(ChatCompletionRequestAssistantMessageArgs::default()
            .content(message.content.clone())
            .build()?
            .into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider_config(api_key: Option<&str>) -> ProviderConfig {
        ProviderConfig {
            api_key: api_key.map(ToString::to_string),
            base_url: "https://api.deepseek.com".to_string(),
            model: "deepseek-chat".to_string(),
            temperature: 0.0,
            max_tokens: 8192,
        }
    }

    #[test]
    fn requires_api_key() {
        let error =
            match OpenAiCompatibleClient::new(ModelProvider::DeepSeek, provider_config(None)) {
                Ok(_) => panic!("client should require api key"),
                Err(error) => error,
            };

        assert!(error.to_string().contains("api key"));
    }

    #[test]
    fn creates_client_with_api_key() {
        let client =
            OpenAiCompatibleClient::new(ModelProvider::DeepSeek, provider_config(Some("test-key")))
                .expect("client should build");

        assert_eq!(client.provider(), ModelProvider::DeepSeek);
    }

    #[test]
    fn converts_user_message() {
        let message = LlmMessage::user("hello");
        let openai_message = to_openai_message(&message).expect("message should convert");

        assert!(matches!(
            openai_message,
            ChatCompletionRequestMessage::User(_)
        ));
    }

    #[test]
    fn converts_system_message() {
        let message = LlmMessage::system("system");
        let openai_message = to_openai_message(&message).expect("message should convert");

        assert!(matches!(
            openai_message,
            ChatCompletionRequestMessage::System(_)
        ));
    }

    #[test]
    fn converts_assistant_message() {
        let message = LlmMessage::assistant("assistant");
        let openai_message = to_openai_message(&message).expect("message should convert");

        assert!(matches!(
            openai_message,
            ChatCompletionRequestMessage::Assistant(_)
        ));
    }
}
