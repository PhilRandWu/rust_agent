use crate::llm::client::LlmClient;
use crate::llm::message::{LlmMessage, LlmResponse};
use std::sync::{Arc, Mutex};

pub struct MockLlmClient {
    responses: Arc<Mutex<Vec<String>>>,
}

impl MockLlmClient {
    pub fn new(response: impl Into<String>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(vec![response.into()])),
        }
    }

    pub fn sequence(responses: Vec<impl Into<String>>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(
                responses.into_iter().map(|s| s.into()).collect(),
            )),
        }
    }
}

impl Default for MockLlmClient {
    fn default() -> Self {
        Self::new(
            serde_json::json!({
                "app_type": "web_app",
                "summary": "Mock analysis for local development",
                "features": ["mock feature"],
                "pages": ["home"],
                "components": ["MockComponent"]
            })
            .to_string(),
        )
    }
}

#[async_trait::async_trait]
impl LlmClient for MockLlmClient {
    async fn chat(&self, _messages: &[LlmMessage]) -> anyhow::Result<LlmResponse> {
        let mut responses = self
            .responses
            .lock()
            .map_err(|_| anyhow::anyhow!("mock llm response lock poisoned"))?;

        let response = if responses.len() > 1 {
            responses.remove(0)
        } else {
            responses
                .first()
                .cloned()
                .unwrap_or_else(|| "Mock LLM Response".to_string())
        };
        Ok(LlmResponse::new(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn returns_configured_mock_response() {
        let client = MockLlmClient::new("mocked");

        let response = client.chat(&[LlmMessage::user("hello")]).await.unwrap();

        assert_eq!(response.content, "mocked");
    }

    #[tokio::test]
    async fn returns_sequence_responses_in_order() {
        let client = MockLlmClient::sequence(vec!["first", "second"]);

        let first = client.chat(&[LlmMessage::user("hello")]).await.unwrap();
        let second = client.chat(&[LlmMessage::user("hello")]).await.unwrap();
        let third = client.chat(&[LlmMessage::user("hello")]).await.unwrap();

        assert_eq!(first.content, "first");
        assert_eq!(second.content, "second");
        assert_eq!(third.content, "second");
    }
}
