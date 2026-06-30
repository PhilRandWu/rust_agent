use crate::agent::analysis::AnalysisOutput;
use crate::llm::client::LlmClient;
use crate::llm::message::LlmMessage;
use crate::llm::structured::structured_chat;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanInput {
    pub analysis: AnalysisOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanOutput {
    pub project_name: String,
    pub description: String,
    pub pages: Vec<PagePlan>,
    pub components: Vec<ComponentPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PagePlan {
    pub name: String,
    pub route: String,
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComponentPlan {
    pub name: String,
    pub purpose: String,
}

pub struct PlanNode {
    client: Arc<dyn LlmClient>,
}

impl PlanInput {
    pub fn new(analysis: AnalysisOutput) -> Self {
        Self { analysis }
    }
}

impl PlanNode {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    pub async fn run(&self, input: PlanInput) -> anyhow::Result<PlanOutput> {
        let messages = plan_messages(&input);
        structured_chat(self.client.as_ref(), &messages).await
    }
}

fn plan_messages(input: &PlanInput) -> Vec<LlmMessage> {
    let analysis_json = serde_json::to_string(&input.analysis).unwrap_or_else(|_| "{}".to_string());

    vec![
        LlmMessage::system(
            r#"You are a frontend application planner.
Based on the analysis JSON, create a concise implementation plan.
Return only valid JSON with these fields:
project_name: string
description: string
pages: [{ name: string, route: string, purpose: string }]
components: [{ name: string, purpose: string }]"#,
        ),
        LlmMessage::user(format!(
            "Create an implementation plan from this analysis JSON: {}",
            analysis_json
        )),
    ]
}

#[cfg(test)]
mod tests {
    use crate::agent::analysis::AnalysisOutput;
    use crate::agent::plan::{PlanInput, PlanNode};
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
