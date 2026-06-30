use crate::agent::analysis::{AnalysisInput, AnalysisNode};
use crate::agent::context::AgentContext;
use crate::agent::event::AgentEvent;
use crate::agent::files::{FilesInput, FilesNode};
use crate::agent::plan::{PlanInput, PlanNode};
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

        let analysis_output = match analysis_node.run(analysis_input).await {
            Ok(output) => output,
            Err(error) => {
                return vec![AgentEvent::Error(error.to_string()), AgentEvent::Done];
            }
        };

        let plan_node = PlanNode::new(Arc::clone(&self.main_client));
        let plan_output = match plan_node.run(PlanInput::new(analysis_output.clone())).await {
            Ok(output) => output,
            Err(error) => {
                return vec![
                    AgentEvent::Analysis(analysis_output),
                    AgentEvent::Error(error.to_string()),
                    AgentEvent::Done,
                ];
            }
        };

        let files_node = FilesNode::new();
        let files_output = match files_node.run(FilesInput::new(plan_output.clone())).await {
            Ok(output) => output,
            Err(error) => {
                return vec![
                    AgentEvent::Analysis(analysis_output),
                    AgentEvent::Plan(plan_output),
                    AgentEvent::Error(error.to_string()),
                    AgentEvent::Done,
                ];
            }
        };

        vec![
            AgentEvent::Analysis(analysis_output),
            AgentEvent::Plan(plan_output),
            AgentEvent::Files(files_output),
            AgentEvent::Done,
        ]
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
    async fn returns_analysis_error_and_done_when_planning_fails() {
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
            [
                AgentEvent::Analysis(output),
                AgentEvent::Error(message),
                AgentEvent::Done,
            ] => {
                assert_eq!(output.app_type, "todo");
                assert_eq!(output.summary, "A todo app");
                assert!(!message.is_empty());
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

    #[tokio::test]
    async fn returns_analysis_plan_and_done_events() {
        let client = Arc::new(MockLlmClient::sequence(vec![
            r#"{
            "app_type": "todo",
            "summary": "A todo app",
            "features": ["create todos"],
            "pages": ["home"],
            "components": ["TodoList"]
        }"#,
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
                }
            ]
        }"#,
        ]));
        let runner = AgentRunner::new(client);

        let events = runner
            .run(AgentContext {
                project_id: Some("project-1".to_string()),
                mock_config: None,
                user_request: "build a todo app".to_string(),
            })
            .await;

        match events.as_slice() {
            [
                AgentEvent::Analysis(analysis),
                AgentEvent::Plan(plan),
                AgentEvent::Files(files),
                AgentEvent::Done,
            ] => {
                assert_eq!(analysis.app_type, "todo");
                assert_eq!(plan.project_name, "Todo App");
                assert!(files.files.contains_key("package.json"));
                assert!(files.files.contains_key("src/App.tsx"));
            }
            other => panic!("unexpected events: {other:?}"),
        }
    }
}
