use crate::llm::client::LlmClient;
use crate::llm::message::LlmMessage;
use crate::llm::structured::structured_chat;
use crate::routes::chat::dto::ChatRequest;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AnalysisInput {
    pub user_request: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnalysisOutput {
    pub app_type: String,
    pub summary: String,
    pub features: Vec<String>,
    pub pages: Vec<String>,
    pub components: Vec<String>,
}

pub struct AnalysisNode {
    client: Arc<dyn LlmClient>,
}

impl AnalysisInput {
    pub fn new(user_request: impl Into<String>) -> Self {
        Self {
            user_request: user_request.into(),
        }
    }

    pub fn from_chat_request(request: &ChatRequest) -> anyhow::Result<Self> {
        let user_request = request
            .messages
            .iter()
            .rev()
            .find(|message| message.role == "user")
            .and_then(|message| message.content.as_str())
            .filter(|content| !content.is_empty())
            .ok_or_else(|| anyhow::anyhow!("chat request must contain a user message"))?;

        Ok(Self::new(user_request))
    }
}

impl AnalysisNode {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    pub async fn run(self, input: AnalysisInput) -> anyhow::Result<AnalysisOutput> {
        let messages = analysis_message(&input);
        structured_chat(self.client.as_ref(), &messages).await
    }
}

fn analysis_message(input: &AnalysisInput) -> Vec<LlmMessage> {
    vec![
        LlmMessage::system(
            r#"You are an app generation requirement analyst.
Return only valid JSON with these fields:
app_type: string
summary: string
features: string[]
pages: string[]
components: string[]"#,
        ),
        LlmMessage::user(format!("Analyze this app request: {}", input.user_request)),
    ]
}

#[cfg(test)]
mod tests {
    use crate::agent::analysis::{AnalysisInput, AnalysisNode};
    use crate::llm::mock::MockLlmClient;
    use crate::routes::chat::dto::{ChatMessage, ChatRequest};
    use serde_json::json;
    use std::sync::Arc;

    #[tokio::test]
    async fn parses_analysis_output_from_llm_json() {
        let client = Arc::new(MockLlmClient::new(
            r#"{
                "app_type": "todo",
                "summary": "A todo management app",
                "features": ["create todos", "complete todos"],
                "pages": ["home"],
                "components": ["TodoList", "TodoInput"]
            }"#,
        ));
        let node = AnalysisNode::new(client);

        let output = node
            .run(AnalysisInput::new("Build a todo app"))
            .await
            .expect("Failed to run analysis node");

        println!("{:?}", output);
        assert_eq!(output.app_type, "todo");
        assert_eq!(output.summary, "A todo management app");
        assert_eq!(output.features, vec!["create todos", "complete todos"]);
        assert_eq!(output.pages, vec!["home"]);
        assert_eq!(output.components, vec!["TodoList", "TodoInput"]);
    }

    #[test]
    fn builds_analysis_input_from_last_user_message() {
        let request = ChatRequest {
            messages: vec![
                ChatMessage {
                    id: Some("1".to_string()),
                    role: "user".to_string(),
                    content: json!("First request"),
                    attachments: None,
                },
                ChatMessage {
                    id: Some("2".to_string()),
                    role: "assistant".to_string(),
                    content: json!("Question"),
                    attachments: None,
                },
                ChatMessage {
                    id: Some("3".to_string()),
                    role: "user".to_string(),
                    content: json!("Build a todo app"),
                    attachments: None,
                },
            ],
            project_id: Some("demo".to_string()),
            mock_config: None,
        };

        let input =
            AnalysisInput::from_chat_request(&request).expect("Failed to create analysis input");
        assert_eq!(input.user_request, "Build a todo app");
    }
}
