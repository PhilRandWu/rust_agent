use crate::agent::flows::traditional::plan::prompt::plan_messages;
use crate::agent::flows::traditional::plan::schema::{PlanInput, PlanOutput};
use crate::llm::client::LlmClient;
use crate::llm::structured::structured_chat;
use std::sync::Arc;

pub struct PlanNode {
    client: Arc<dyn LlmClient>,
}

impl PlanNode {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    #[tracing::instrument(skip_all, name = "node.plan")]
    pub async fn run(&self, input: PlanInput) -> anyhow::Result<PlanOutput> {
        let messages = plan_messages(&input);
        structured_chat(self.client.as_ref(), &messages).await
    }
}

#[cfg(test)]
mod tests {
    use crate::agent::flows::traditional::analysis::schema::AnalysisOutput;
    use crate::agent::flows::traditional::plan::node::PlanNode;
    use crate::agent::flows::traditional::plan::schema::PlanInput;
    use crate::llm::mock::MockLlmClient;
    use std::sync::Arc;

    #[tokio::test]
    async fn parses_plan_output_from_llm_json() {
        let client = Arc::new(MockLlmClient::new(
            r#"{
                "project_name": "Todo App",
                "description": "A todo management application",
                "pages": [
                    {
                        "name": "Home",
                        "route": "/",
                        "purpose": "Display and manage todos"
                    }
                ],
                "components": [
                    {
                        "name": "TodoList",
                        "purpose": "Render todo items"
                    },
                    {
                        "name": "TodoInput",
                        "purpose": "Create new todo items"
                    }
                ]
            }"#,
        ));

        let node = PlanNode::new(client);
        let output = node
            .run(PlanInput::new(AnalysisOutput {
                app_type: "todo".to_string(),
                summary: "A todo management app".to_string(),
                features: vec!["create todos".to_string(), "complete todos".to_string()],
                pages: vec!["home".to_string()],
                components: vec!["TodoList".to_string(), "TodoInput".to_string()],
            }))
            .await
            .expect("Should succeed");

        assert_eq!(output.project_name, "Todo App");
        assert_eq!(output.pages.len(), 1);
        assert_eq!(output.pages[0].route, "/");
        assert_eq!(output.components.len(), 2);
        assert_eq!(output.components[0].name, "TodoList");
    }
}
