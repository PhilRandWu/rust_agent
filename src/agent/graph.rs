use crate::agent::context::AgentContext;
use crate::agent::event::AgentEvent;
use crate::agent::mock::MockStore;
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

    pub fn with_mock_store(main_client: Arc<dyn LlmClient>, mock_store: MockStore) -> Self {
        Self {
            runner: AgentRunner::with_mock_store(main_client, mock_store),
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
    use crate::agent::mock::MockStore;
    use crate::llm::mock::MockLlmClient;
    use serde_json::json;
    use std::sync::Arc;

    fn all_nodes_mock_config() -> serde_json::Value {
        json!({
            "analysisNode": true,
            "intentNode": true,
            "capabilityNode": true,
            "uiNode": true,
            "componentNode": true,
            "structureNode": true,
            "dependencyNode": true,
            "typeNode": true,
            "utilsNode": true,
            "mockDataNode": true,
            "serviceNode": true,
            "hooksNode": true,
            "componentGenNode": true,
            "pageGenNode": true,
            "layoutNode": true,
            "styleGenNode": true,
            "appGenNode": true,
            "assembleNode": true,
            "postProcessNode": true
        })
    }

    #[tokio::test]
    async fn traditional_graph_runs_full_pipeline_with_mock_store() {
        let client = Arc::new(MockLlmClient::default());
        let graph = TraditionalGraph::with_mock_store(client, MockStore::new("mock"));

        let events = graph
            .run(AgentContext {
                project_id: Some("project-1".to_string()),
                mock_config: Some(all_nodes_mock_config()),
                user_request: "build a todo app".to_string(),
            })
            .await;

        assert!(
            events
                .iter()
                .any(|event| matches!(event, AgentEvent::Analysis(_))),
            "expected Analysis event, got: {events:?}"
        );
        assert!(
            events
                .iter()
                .any(|event| matches!(event, AgentEvent::Intent(_))),
            "expected Intent event, got: {events:?}"
        );
        assert!(
            events
                .iter()
                .any(|event| matches!(event, AgentEvent::PostProcess(_))),
            "expected PostProcess event, got: {events:?}"
        );

        match events.as_slice() {
            [.., AgentEvent::Files(files), AgentEvent::Done] => {
                assert!(!files.files.is_empty(), "expected non-empty files");
            }
            other => panic!("unexpected trailing events: {other:?}"),
        }
    }
}
