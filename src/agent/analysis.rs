use crate::llm::client::LlmClient;
use crate::llm::message::LlmMessage;
use crate::llm::structured::structured_chat;
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
    use crate::agent::event::AgentEvent;
    use crate::llm::mock::MockLlmClient;
    use crate::routes::chat::dto::{ChatMessage, ChatRequest};
    use crate::sse::event::FrontendEvent;
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
    fn converts_agent_done_event_to_frontend_event() {
        let event = FrontendEvent::from_agent_event(AgentEvent::Done);

        let data = event.to_sse_data().unwrap();

        assert_eq!(data, r#"{"type":"done"}"#);
    }

    #[test]
    fn converts_agent_error_event_to_frontend_event() {
        let event = FrontendEvent::from_agent_event(AgentEvent::Error("failed".to_string()));

        let data = event.to_sse_data().unwrap();

        assert_eq!(data, r#"{"type":"error","message":"failed"}"#);
    }
}
