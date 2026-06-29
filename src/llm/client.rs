use crate::llm::message::{LlmMessage, LlmResponse};

#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    async fn chat(&self, messages: &[LlmMessage]) -> anyhow::Result<LlmResponse>;
}
