use crate::agent::context::AgentContext;
use crate::agent::event::AgentEvent;
use crate::agent::runner::AgentRunner;
use crate::llm::client::LlmClient;
use std::sync::Arc;

#[async_trait::async_trait]
pub trait AgentGraph: Send + Sync {
    async fn run(&self, context: AgentContext) -> Vec<AgentEvent>;
}

pub struct TraditionalGraph {
    runner: AgentRunner,
}

impl TraditionalGraph {
    pub fn new(main_client: Arc<dyn LlmClient>) -> Self {
        Self {
            runner: AgentRunner::new(main_client),
        }
    }
}

#[async_trait::async_trait]
impl AgentGraph for TraditionalGraph {
    async fn run(&self, context: AgentContext) -> Vec<AgentEvent> {
        self.runner.run(context).await
    }
}

#[cfg(test)]
mod tests {
    use crate::agent::context::AgentContext;
    use crate::agent::event::AgentEvent;
    use crate::agent::graph::{AgentGraph, TraditionalGraph};
    use crate::llm::mock::MockLlmClient;
    use std::sync::Arc;

    #[tokio::test]
    async fn traditional_graph_returns_analysis_plan_files_and_done() {
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
                        "purpose": "Display todos"
                    }
                ],
                "components": [
                    {
                        "name": "TodoList",
                        "purpose": "Render todos"
                    }
                ]
            }"#,
        ]));
        let graph = TraditionalGraph::new(client);
        let events = graph
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
