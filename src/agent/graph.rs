use crate::agent::context::AgentContext;
use crate::agent::event::AgentEvent;
use crate::agent::mock::MockStore;
use crate::agent::runner::AgentRunner;
use crate::llm::client::LlmClient;
use crate::llm::semaphore::LlmGate;
use crate::session::SessionStore;
use std::sync::Arc;
use tokio::sync::mpsc;

#[async_trait::async_trait]
pub trait AgentGraph: Send + Sync {
    async fn run_streaming(&self, context: AgentContext, tx: mpsc::Sender<AgentEvent>);
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

    pub fn with_gate(main_client: Arc<dyn LlmClient>, gate: LlmGate) -> Self {
        Self {
            runner: AgentRunner::with_gate(main_client, gate),
        }
    }

    pub fn with_mock_store_and_gate(
        main_client: Arc<dyn LlmClient>,
        mock_store: MockStore,
        gate: LlmGate,
    ) -> Self {
        Self {
            runner: AgentRunner::with_mock_store_and_gate(main_client, mock_store, gate),
        }
    }

    pub fn with_gate_and_session(
        main_client: Arc<dyn LlmClient>,
        gate: LlmGate,
        session_store: Arc<dyn SessionStore>,
    ) -> Self {
        Self {
            runner: AgentRunner::with_gate(main_client, gate).with_session_store(session_store),
        }
    }
}

#[async_trait::async_trait]
impl AgentGraph for TraditionalGraph {
    async fn run_streaming(&self, context: AgentContext, tx: mpsc::Sender<AgentEvent>) {
        self.runner.run_streaming(context, tx).await
    }
}

pub async fn collect_events<G>(graph: &G, context: AgentContext) -> Vec<AgentEvent>
where
    G: AgentGraph + ?Sized,
{
    let (tx, mut rx) = mpsc::channel(32);

    let producer = async {
        graph.run_streaming(context, tx).await;
    };
    let consumer = async {
        let mut events = Vec::new();
        while let Some(evt) = rx.recv().await {
            events.push(evt);
        }
        events
    };

    let (_, events) = tokio::join!(producer, consumer);
    events
}

#[cfg(test)]
mod tests {
    use crate::agent::context::AgentContext;
    use crate::agent::event::AgentEvent;
    use crate::agent::graph::{TraditionalGraph, collect_events};
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
            "appGenNode": true
        })
    }

    #[tokio::test]
    async fn traditional_graph_runs_full_pipeline_with_mock_store() {
        let client = Arc::new(MockLlmClient::default());
        let graph = TraditionalGraph::with_mock_store(client, MockStore::new("mock"));

        let events = collect_events(
            &graph,
            AgentContext {
                project_id: Some("project-1".to_string()),
                mock_config: Some(all_nodes_mock_config()),
                user_request: "build a todo app".to_string(),
            },
        )
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
            [.., AgentEvent::PostProcess(post_process), AgentEvent::Done] => {
                assert!(
                    !post_process.files.is_empty(),
                    "expected non-empty post-process files"
                );
            }
            other => panic!("unexpected trailing events: {other:?}"),
        }
    }
}
