use crate::agent::analysis::{AnalysisInput, AnalysisNode};
use crate::agent::context::AgentContext;
use crate::agent::event::AgentEvent;
use crate::llm::client::LlmClient;
use std::sync::Arc;

pub struct AgentRunner {
    main_client: Arc<dyn LlmClient>,
}

impl AgentRunner {
    pub fn new(main_client: Arc<dyn LlmClient>) -> Self {
        Self { main_client }
    }

    pub async fn run(&self, context: AgentContext) -> Vec<AgentEvent> {
        let analysis_input = AnalysisInput::new(context.user_request);
        let analysis_node = AnalysisNode::new(Arc::clone(&self.main_client));

        match analysis_node.run(analysis_input).await {
            Ok(output) => vec![AgentEvent::Analysis(output), AgentEvent::Done],
            Err(error) => vec![AgentEvent::Error(error.to_string()), AgentEvent::Done],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::agent::context::AgentContext;
    use crate::agent::event::AgentEvent;
    use crate::agent::runner::AgentRunner;
    use crate::llm::mock::MockLlmClient;
    use std::sync::Arc;

    #[tokio::test]
    async fn returns_analysis_and_done_events() {
        let client = Arc::new(MockLlmClient::new(
            r#"{
                "app_type": "todo",
                "summary": "A todo app",
                "features": ["create todos"],
                "pages": ["home"],
                "components": ["TodoList"]
            }"#,
        ));

        let runner = AgentRunner::new(client);
        let events = runner
            .run(AgentContext {
                project_id: Some("project-1".to_string()),
                mock_config: None,
                user_request: "build a todo app".to_string(),
            })
            .await;

        match events.as_slice() {
            [AgentEvent::Analysis(output), AgentEvent::Done] => {
                assert_eq!(output.app_type, "todo");
                assert_eq!(output.summary, "A todo app");
            }
            other => panic!("unexpected events: {:?}", other),
        }
    }

    #[tokio::test]
    async fn returns_error_and_done_when_analysis_fails() {
        let client = Arc::new(MockLlmClient::new("not json"));
        let runner = AgentRunner::new(client);

        let events = runner
            .run(AgentContext {
                project_id: None,
                mock_config: None,
                user_request: "build a todo app".to_string(),
            })
            .await;

        match events.as_slice() {
            [AgentEvent::Error(message), AgentEvent::Done] => {
                assert!(!message.is_empty());
            }
            other => panic!("unexpected events: {other:?}"),
        }
    }
}
