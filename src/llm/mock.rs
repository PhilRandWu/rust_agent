use crate::llm::client::LlmClient;
use crate::llm::message::{LlmMessage, LlmResponse};

pub struct MockLlmClient {
    response: String,
}

impl MockLlmClient {
    pub fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
        }
    }
}

impl Default for MockLlmClient {
    fn default() -> Self {
        Self {
            response: serde_json::json!({
                "app_type": "web_app",
                "summary": "Mock analysis for local development",
                "features": ["mock feature"],
                "pages": ["home"],
                "components": ["MockComponent"]
            })
            .to_string(),
        }
    }
}

#[async_trait::async_trait]
impl LlmClient for MockLlmClient {
    async fn chat(&self, _messages: &[LlmMessage]) -> anyhow::Result<LlmResponse> {
        Ok(LlmResponse::new(self.response.clone()))
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
}
