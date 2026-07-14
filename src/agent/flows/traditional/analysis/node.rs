use crate::agent::flows::traditional::analysis::prompt::analysis_message;
use crate::agent::flows::traditional::analysis::schema::{AnalysisInput, AnalysisOutput};
use crate::llm::client::LlmClient;
use crate::llm::structured::structured_chat;
use std::sync::Arc;

pub struct AnalysisNode {
    client: Arc<dyn LlmClient>,
}

impl AnalysisNode {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    #[tracing::instrument(skip_all, name = "node.analysis")]
    pub async fn run(&self, input: AnalysisInput) -> anyhow::Result<AnalysisOutput> {
        let messages = analysis_message(&input);
        structured_chat(self.client.as_ref(), &messages).await
    }
}

#[cfg(test)]
mod tests {
    use crate::agent::flows::traditional::analysis::node::AnalysisNode;
    use crate::agent::flows::traditional::analysis::schema::AnalysisInput;
    use crate::llm::mock::MockLlmClient;
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

        assert_eq!(output.app_type, "todo");
        assert_eq!(output.summary, "A todo management app");
        assert_eq!(output.features, vec!["create todos", "complete todos"]);
        assert_eq!(output.pages, vec!["home"]);
        assert_eq!(output.components, vec!["TodoList", "TodoInput"]);
    }
}
