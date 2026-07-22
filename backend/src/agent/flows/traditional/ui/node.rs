use crate::agent::flows::traditional::ui::prompt::ui_messages;
use crate::agent::flows::traditional::ui::{UiInput, UiOutput};
use crate::llm::client::LlmClient;
use crate::llm::structured::structured_chat;
use std::sync::Arc;

pub struct UiNode {
    client: Arc<dyn LlmClient>,
}

impl UiNode {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    #[tracing::instrument(skip_all, name = "node.ui")]
    pub async fn run(&self, input: UiInput) -> anyhow::Result<UiOutput> {
        let messages = ui_messages(&input);
        structured_chat(self.client.as_ref(), &messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::analysis::AnalysisOutput;
    use crate::agent::flows::traditional::capability::schema::CapabilityPriority;
    use crate::agent::flows::traditional::capability::{CapabilityItem, CapabilityOutput};
    use crate::agent::flows::traditional::intent::{IntentOutput, IntentType};
    use crate::agent::flows::traditional::ui::schema::{LayoutPattern, UiInput, VisualStyle};
    use crate::llm::mock::MockLlmClient;

    #[tokio::test]
    async fn parses_ui_output_from_llm_json() {
        let client = Arc::new(MockLlmClient::new(
            r#"{
                "layout_pattern": "SINGLE_PAGE",
                "visual_style": "MODERN",
                "primary_sections": ["Hero", "Todo input", "Todo list"],
                "interactions": ["Create todo", "Mark todo complete"],
                "responsive_notes": ["Stack content vertically on mobile"]
            }"#,
        ));

        let node = UiNode::new(client);

        let output = node
            .run(UiInput::new(
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
                    constraints: vec!["Keep UI simple".to_string()],
                    user_stories: vec!["As a user, I can create todos".to_string()],
                },
                CapabilityOutput {
                    capabilities: vec![CapabilityItem {
                        name: "Todo creation".to_string(),
                        description: "Users can create todo items".to_string(),
                        priority: CapabilityPriority::MustHave,
                    }],
                },
            ))
            .await
            .expect("ui node should parse output");

        assert_eq!(output.layout_pattern, LayoutPattern::SinglePage);
        assert_eq!(output.visual_style, VisualStyle::Modern);
        assert_eq!(output.primary_sections.len(), 3);
        assert_eq!(output.interactions[0], "Create todo");
    }
}
