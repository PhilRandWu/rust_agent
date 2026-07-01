use crate::agent::flows::traditional::intent::prompt::intent_messages;
use crate::agent::flows::traditional::intent::{IntentInput, IntentOutput};
use crate::llm::client::LlmClient;
use crate::llm::structured::structured_chat;
use std::sync::Arc;

pub struct IntentNode {
    client: Arc<dyn LlmClient>,
}

impl IntentNode {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    pub async fn run(&self, input: IntentInput) -> anyhow::Result<IntentOutput> {
        let messages = intent_messages(&input);
        structured_chat(self.client.as_ref(), &messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::analysis::AnalysisOutput;
    use crate::agent::flows::traditional::intent::schema::{IntentInput, IntentType};
    use crate::llm::mock::MockLlmClient;

    #[tokio::test]
    async fn parses_intent_output_from_llm_json() {
        let client = Arc::new(MockLlmClient::new(
            r#"{
                "intent_type": "CREATE",
                "goal": "Create a todo management app",
                "constraints": ["Use React", "Keep UI simple"],
                "user_stories": [
                    "As a user, I can create todos",
                    "As a user, I can mark todos complete"
                ]
            }"#,
        ));

        let node = IntentNode::new(client);

        let output = node
            .run(IntentInput::new(
                AnalysisOutput {
                    app_type: "todo".to_string(),
                    summary: "A todo app".to_string(),
                    features: vec!["create todos".to_string()],
                    pages: vec!["home".to_string()],
                    components: vec!["TodoList".to_string()],
                },
                "build a todo app",
            ))
            .await
            .expect("intent node should parse output");

        assert_eq!(output.intent_type, IntentType::Create);
        assert_eq!(output.goal, "Create a todo management app");
        assert_eq!(output.constraints, vec!["Use React", "Keep UI simple"]);
        assert_eq!(output.user_stories.len(), 2);
    }
}
