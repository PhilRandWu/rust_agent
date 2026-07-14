use crate::agent::flows::traditional::capability::prompt::capability_messages;
use crate::agent::flows::traditional::capability::{CapabilityInput, CapabilityOutput};
use crate::llm::client::LlmClient;
use crate::llm::structured::structured_chat;
use std::sync::Arc;

pub struct CapabilityNode {
    client: Arc<dyn LlmClient>,
}

impl CapabilityNode {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    #[tracing::instrument(skip_all, name = "node.capability")]
    pub async fn run(&self, input: CapabilityInput) -> anyhow::Result<CapabilityOutput> {
        let messages = capability_messages(&input);
        structured_chat(self.client.as_ref(), &messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::analysis::AnalysisOutput;
    use crate::agent::flows::traditional::capability::schema::{
        CapabilityInput, CapabilityPriority,
    };
    use crate::agent::flows::traditional::intent::{IntentOutput, IntentType};
    use crate::llm::mock::MockLlmClient;

    #[tokio::test]
    async fn parses_capability_output_from_llm_json() {
        let client = Arc::new(MockLlmClient::new(
            r#"{
                "capabilities": [
                    {
                        "name": "Todo creation",
                        "description": "Users can create todo items",
                        "priority": "MUST_HAVE"
                    },
                    {
                        "name": "Todo completion",
                        "description": "Users can mark todo items as complete",
                        "priority": "SHOULD_HAVE"
                    }
                ]
            }"#,
        ));

        let node = CapabilityNode::new(client);

        let output = node
            .run(CapabilityInput::new(
                AnalysisOutput {
                    app_type: "todo".to_string(),
                    summary: "A todo app".to_string(),
                    features: vec!["create todos".to_string()],
                    pages: vec!["home".to_string()],
                    components: vec!["TodoList".to_string()],
                },
                IntentOutput {
                    intent_type: IntentType::Create,
                    goal: "Create a todo app".to_string(),
                    constraints: vec!["Use React".to_string()],
                    user_stories: vec!["As a user, I can create todos".to_string()],
                },
            ))
            .await
            .expect("capability node should parse output");

        assert_eq!(output.capabilities.len(), 2);
        assert_eq!(output.capabilities[0].name, "Todo creation");
        assert_eq!(
            output.capabilities[0].priority,
            CapabilityPriority::MustHave
        );
    }
}
