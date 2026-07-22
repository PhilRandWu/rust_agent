use crate::agent::context::AgentContext;
use crate::agent::event::AgentEvent;
use crate::agent::flow_state::TraditionalFlowState;
use crate::agent::flows::traditional::analysis::{AnalysisInput, AnalysisNode};
use crate::agent::flows::traditional::app_gen::{AppGenInput, AppGenNode};
use crate::agent::flows::traditional::assemble::{AssembleInput, AssembleNode};
use crate::agent::flows::traditional::capability::{CapabilityInput, CapabilityNode};
use crate::agent::flows::traditional::component::{ComponentInput, ComponentNode};
use crate::agent::flows::traditional::component_gen::{
    ComponentGenInput, ComponentGenNode, ComponentGenOutput,
};
use crate::agent::flows::traditional::dependency::{DependencyInput, DependencyNode};
use crate::agent::flows::traditional::hooks::{HooksInput, HooksNode};
use crate::agent::flows::traditional::intent::{IntentInput, IntentNode};
use crate::agent::flows::traditional::layout::{LayoutInput, LayoutNode};
use crate::agent::flows::traditional::mock_data::{MockDataInput, MockDataNode};
use crate::agent::flows::traditional::page_gen::{PageGenInput, PageGenNode};
use crate::agent::flows::traditional::post_process::{PostProcessInput, PostProcessNode};
use crate::agent::flows::traditional::service::{ServiceInput, ServiceNode};
use crate::agent::flows::traditional::structure::{StructureInput, StructureNode};
use crate::agent::flows::traditional::style_gen::{StyleGenInput, StyleGenNode};
use crate::agent::flows::traditional::typegen::{TypeInput, TypeNode};
use crate::agent::flows::traditional::ui::{UiInput, UiNode};
use crate::agent::flows::traditional::utils::{UtilsInput, UtilsNode};
use crate::agent::mock::{MockNode, MockStore};
use crate::llm::client::LlmClient;
use crate::llm::semaphore::LlmGate;
use crate::session::{Operation, SessionStore, VersionMeta, VersionSnapshot, truncate_prompt};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;

pub struct AgentRunner {
    main_client: Arc<dyn LlmClient>,
    mock_store: MockStore,
    gate: LlmGate,
    session_store: Option<Arc<dyn SessionStore>>,
}

impl AgentRunner {
    pub fn new(main_client: Arc<dyn LlmClient>) -> Self {
        Self {
            main_client,
            mock_store: MockStore::default(),
            gate: LlmGate::new(1),
            session_store: None,
        }
    }

    pub fn with_mock_store(main_client: Arc<dyn LlmClient>, mock_store: MockStore) -> Self {
        Self::with_mock_store_and_gate(main_client, mock_store, LlmGate::new(1))
    }

    pub fn with_gate(main_client: Arc<dyn LlmClient>, gate: LlmGate) -> Self {
        Self {
            main_client,
            mock_store: MockStore::default(),
            gate,
            session_store: None,
        }
    }

    pub fn with_mock_store_and_gate(
        main_client: Arc<dyn LlmClient>,
        mock_store: MockStore,
        gate: LlmGate,
    ) -> Self {
        Self {
            main_client,
            mock_store,
            gate,
            session_store: None,
        }
    }

    pub fn with_session_store(mut self, store: Arc<dyn SessionStore>) -> Self {
        self.session_store = Some(store);
        self
    }

    pub async fn run_streaming(&self, context: AgentContext, tx: mpsc::Sender<AgentEvent>) {
        use tracing::Instrument;
        let span = tracing::info_span!(
            "traditional_pipeline",
            project_id = context.project_id.as_deref().unwrap_or("-"),
            user_request_len = context.user_request.len(),
            mock = context.mock_config.is_some(),
        );
        let started = std::time::Instant::now();
        let result = self
            .run_traditional(context, &tx)
            .instrument(span.clone())
            .await;
        let elapsed_ms = started.elapsed().as_millis();
        if let Err(error) = result {
            tracing::warn!(target: "pipeline", parent: &span, elapsed_ms, error = %error, "pipeline failed");
            let _ = tx.send(AgentEvent::Error(error.to_string())).await;
        } else {
            tracing::info!(target: "pipeline", parent: &span, elapsed_ms, "pipeline ok");
        }
        let _ = tx.send(AgentEvent::Done).await;
    }

    async fn run_traditional(
        &self,
        context: AgentContext,
        tx: &mpsc::Sender<AgentEvent>,
    ) -> anyhow::Result<()> {
        let mut state = TraditionalFlowState::new();

        let analysis = if self.should_mock(&context, MockNode::Analysis) {
            self.mock_store.load(MockNode::Analysis).await?
        } else {
            let analysis_input = AnalysisInput::new(context.user_request.clone());
            let analysis_node = AnalysisNode::new(Arc::clone(&self.main_client));

            analysis_node.run(analysis_input).await?
        };
        send(tx, AgentEvent::Analysis(analysis.clone())).await?;
        state.analysis = Some(analysis.clone());

        let intent = if self.should_mock(&context, MockNode::Intent) {
            self.mock_store.load(MockNode::Intent).await?
        } else {
            let intent_input = IntentInput::new(analysis.clone(), context.user_request.clone());
            let intent_node = IntentNode::new(Arc::clone(&self.main_client));

            intent_node.run(intent_input).await?
        };

        send(tx, AgentEvent::Intent(intent.clone())).await?;
        state.intent = Some(intent.clone());

        let capability = if self.should_mock(&context, MockNode::Capability) {
            self.mock_store.load(MockNode::Capability).await?
        } else {
            let intent_input = CapabilityInput::new(analysis.clone(), intent.clone());
            let intent_node = CapabilityNode::new(Arc::clone(&self.main_client));

            intent_node.run(intent_input).await?
        };

        send(tx, AgentEvent::Capability(capability.clone())).await?;
        state.capability = Some(capability.clone());

        let ui = if self.should_mock(&context, MockNode::Ui) {
            self.mock_store.load(MockNode::Ui).await?
        } else {
            let ui_input = UiInput::new(analysis.clone(), intent.clone(), capability.clone());
            let ui_node = UiNode::new(Arc::clone(&self.main_client));

            ui_node.run(ui_input).await?
        };

        send(tx, AgentEvent::Ui(ui.clone())).await?;
        state.ui = Some(ui.clone());

        let component = if self.should_mock(&context, MockNode::Component) {
            self.mock_store.load(MockNode::Component).await?
        } else {
            let component_input =
                ComponentInput::new(intent.clone(), capability.clone(), ui.clone());
            let component_node = ComponentNode::new(Arc::clone(&self.main_client));

            component_node.run(component_input).await?
        };

        send(tx, AgentEvent::Component(component.clone())).await?;
        state.component = Some(component.clone());

        let structure = if self.should_mock(&context, MockNode::Structure) {
            self.mock_store.load(MockNode::Structure).await?
        } else {
            let structure_input =
                StructureInput::new(ui.clone(), component.clone(), capability.clone());
            let structure_node = StructureNode::new(Arc::clone(&self.main_client));

            structure_node.run(structure_input).await?
        };

        send(tx, AgentEvent::Structure(structure.clone())).await?;
        state.structure = Some(structure.clone());

        let dependency = if self.should_mock(&context, MockNode::Dependency) {
            self.mock_store.load(MockNode::Dependency).await?
        } else {
            DependencyNode::new()
                .run(DependencyInput::new(structure.clone()))
                .await?
        };

        send(tx, AgentEvent::Dependency(dependency.clone())).await?;
        state.dependency = Some(dependency.clone());

        let types = if self.should_mock(&context, MockNode::Type) {
            self.mock_store.load(MockNode::Type).await?
        } else {
            TypeNode::new(Arc::clone(&self.main_client))
                .run(TypeInput::new(component.clone(), structure.clone()))
                .await?
        };

        send(tx, AgentEvent::Type(types.clone())).await?;
        state.types = Some(types.clone());

        let utils = if self.should_mock(&context, MockNode::Utils) {
            self.mock_store.load(MockNode::Utils).await?
        } else {
            UtilsNode::new()
                .run(UtilsInput::new(structure.clone()))
                .await?
        };

        send(tx, AgentEvent::Utils(utils.clone())).await?;
        state.utils = Some(utils.clone());

        let mock_data = if self.should_mock(&context, MockNode::MockData) {
            self.mock_store.load(MockNode::MockData).await?
        } else {
            MockDataNode::new(Arc::clone(&self.main_client))
                .run(MockDataInput::new(types.clone(), structure.clone()))
                .await?
        };

        send(tx, AgentEvent::MockData(mock_data.clone())).await?;
        state.mock_data = Some(mock_data.clone());

        let service = if self.should_mock(&context, MockNode::Service) {
            self.mock_store.load(MockNode::Service).await?
        } else {
            ServiceNode::new(Arc::clone(&self.main_client))
                .run(ServiceInput::new(
                    types.clone(),
                    mock_data.clone(),
                    utils.clone(),
                    structure.clone(),
                ))
                .await?
        };

        send(tx, AgentEvent::Service(service.clone())).await?;
        state.service = Some(service.clone());

        let hooks = if self.should_mock(&context, MockNode::Hooks) {
            self.mock_store.load(MockNode::Hooks).await?
        } else {
            HooksNode::new(Arc::clone(&self.main_client))
                .run(HooksInput::new(
                    types.clone(),
                    service.clone(),
                    structure.clone(),
                ))
                .await?
        };
        send(tx, AgentEvent::Hooks(hooks.clone())).await?;
        state.hooks = Some(hooks.clone());

        let component_gen = if self.should_mock(&context, MockNode::ComponentGen) {
            self.mock_store.load(MockNode::ComponentGen).await?
        } else {
            let node =
                ComponentGenNode::with_gate(Arc::clone(&self.main_client), self.gate.clone());
            let input = ComponentGenInput::new(
                structure.clone(),
                component.clone(),
                types.clone(),
                service.clone(),
                hooks.clone(),
            );
            match node.stream(input) {
                None => ComponentGenOutput { files: Vec::new() },
                Some(mut stream) => {
                    while let Some(evt) = stream.next_partial().await {
                        send(tx, AgentEvent::ComponentGenPartial(evt)).await?;
                    }
                    ComponentGenOutput {
                        files: stream.finish()?,
                    }
                }
            }
        };
        send(tx, AgentEvent::ComponentGen(component_gen.clone())).await?;
        state.component_gen = Some(component_gen.clone());

        let page_gen = if self.should_mock(&context, MockNode::PageGen) {
            self.mock_store.load(MockNode::PageGen).await?
        } else {
            PageGenNode::new(Arc::clone(&self.main_client))
                .run(PageGenInput::new(
                    structure.clone(),
                    ui.clone(),
                    component.clone(),
                    component_gen.clone(),
                    hooks.clone(),
                    types.clone(),
                ))
                .await?
        };
        send(tx, AgentEvent::PageGen(page_gen.clone())).await?;
        state.page_gen = Some(page_gen.clone());

        let layout = if self.should_mock(&context, MockNode::Layout) {
            self.mock_store.load(MockNode::Layout).await?
        } else {
            LayoutNode::new(Arc::clone(&self.main_client))
                .run(LayoutInput::new(
                    ui.clone(),
                    structure.clone(),
                    page_gen.clone(),
                ))
                .await?
        };
        send(tx, AgentEvent::Layout(layout.clone())).await?;
        state.layout = Some(layout.clone());

        let style_gen = if self.should_mock(&context, MockNode::StyleGen) {
            self.mock_store.load(MockNode::StyleGen).await?
        } else {
            StyleGenNode::new(Arc::clone(&self.main_client))
                .run(StyleGenInput::new(
                    ui.clone(),
                    component.clone(),
                    page_gen.clone(),
                    dependency.clone(),
                ))
                .await?
        };
        send(tx, AgentEvent::StyleGen(style_gen.clone())).await?;
        state.style_gen = Some(style_gen.clone());

        let app_gen = if self.should_mock(&context, MockNode::AppGen) {
            self.mock_store.load(MockNode::AppGen).await?
        } else {
            AppGenNode::new(Arc::clone(&self.main_client))
                .run(AppGenInput::new(
                    ui.clone(),
                    structure.clone(),
                    page_gen.clone(),
                    layout.clone(),
                ))
                .await?
        };
        send(tx, AgentEvent::AppGen(app_gen.clone())).await?;
        state.app_gen = Some(app_gen.clone());

        let assemble = AssembleNode::new()
            .run(AssembleInput::new(
                dependency,
                types,
                utils,
                mock_data,
                service,
                hooks,
                component_gen,
                page_gen,
                layout,
                style_gen,
                app_gen,
            ))
            .await?;
        send(tx, AgentEvent::Assemble(assemble.clone())).await?;
        state.assemble = Some(assemble.clone());

        let post_process = PostProcessNode::new()
            .run(PostProcessInput::new(assemble))
            .await?;
        send(tx, AgentEvent::PostProcess(post_process.clone())).await?;
        state.post_process = Some(post_process.clone());

        self.maybe_commit_session(&context, &post_process.files, tx)
            .await?;

        Ok(())
    }

    async fn maybe_commit_session(
        &self,
        context: &AgentContext,
        files: &std::collections::BTreeMap<String, String>,
        tx: &mpsc::Sender<AgentEvent>,
    ) -> anyhow::Result<()> {
        let Some(store) = self.session_store.as_ref() else {
            return Ok(());
        };
        let Some(project_id) = context.project_id.as_deref() else {
            return Ok(());
        };

        let prev = store.latest_version(project_id).await;
        let operation = if prev == 0 {
            Operation::Create
        } else {
            Operation::Edit
        };
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let snapshot = VersionSnapshot {
            version: 0,
            timestamp_ms,
            operation,
            prompt: truncate_prompt(&context.user_request),
            file_count: files.len(),
            files: files.clone(),
        };
        let assigned = store.append_version(project_id, snapshot).await;
        let meta = VersionMeta {
            version: assigned,
            timestamp_ms,
            operation,
            prompt: truncate_prompt(&context.user_request),
            file_count: files.len(),
        };
        tracing::info!(
            target: "session.commit",
            project_id,
            version = assigned,
            operation = ?operation,
            file_count = files.len(),
            "session committed"
        );
        send(tx, AgentEvent::SessionCommit(meta)).await?;
        Ok(())
    }

    fn should_mock(&self, context: &AgentContext, node: MockNode) -> bool {
        MockStore::should_mock(context.mock_config.as_ref(), node)
    }
}

async fn send(tx: &mpsc::Sender<AgentEvent>, evt: AgentEvent) -> anyhow::Result<()> {
    tx.send(evt)
        .await
        .map_err(|_| anyhow::anyhow!("agent event stream closed by consumer"))
}

#[cfg(test)]
mod tests {
    use crate::agent::context::AgentContext;
    use crate::agent::event::AgentEvent;
    use crate::agent::mock::MockStore;
    use crate::agent::runner::AgentRunner;
    use crate::llm::mock::MockLlmClient;
    use serde_json::json;
    use std::sync::Arc;
    use tokio::sync::mpsc;
    use tokio::time::{Duration, timeout};

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

    async fn collect(runner: &AgentRunner, context: AgentContext) -> Vec<AgentEvent> {
        let (tx, mut rx) = mpsc::channel(32);

        let producer = async {
            runner.run_streaming(context, tx).await;
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

    #[tokio::test]
    async fn returns_analysis_error_and_done_when_intent_fails() {
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
        let events = collect(
            &runner,
            AgentContext {
                project_id: Some("project-1".to_string()),
                mock_config: Some(json!({ "analysisNode": true })),
                user_request: "build a todo app".to_string(),
            },
        )
        .await;

        match events.as_slice() {
            [
                AgentEvent::Analysis(output),
                AgentEvent::Error(message),
                AgentEvent::Done,
            ] => {
                assert_eq!(output.app_type, "todo");
                assert_eq!(output.summary, "Mock todo app analysis");
                assert!(!message.is_empty());
            }
            other => panic!("unexpected events: {:?}", other),
        }
    }

    #[tokio::test]
    async fn returns_error_and_done_when_analysis_fails() {
        let client = Arc::new(MockLlmClient::new("not json"));
        let runner = AgentRunner::new(client);

        let events = collect(
            &runner,
            AgentContext {
                project_id: None,
                mock_config: None,
                user_request: "build a todo app".to_string(),
            },
        )
        .await;

        match events.as_slice() {
            [AgentEvent::Error(message), AgentEvent::Done] => {
                assert!(!message.is_empty());
            }
            other => panic!("unexpected events: {other:?}"),
        }
    }

    #[tokio::test]
    async fn runs_full_traditional_pipeline_with_mock_store() {
        let client = Arc::new(MockLlmClient::default());
        let runner = AgentRunner::with_mock_store(client, MockStore::new("mock"));

        let events = collect(
            &runner,
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
                .any(|event| matches!(event, AgentEvent::Capability(_))),
            "expected Capability event, got: {events:?}"
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
                assert!(
                    post_process.files.contains_key("/pages/Home.tsx"),
                    "expected /pages/Home.tsx in final files, got keys: {:?}",
                    post_process.files.keys().collect::<Vec<_>>()
                );
                assert!(
                    post_process.files.contains_key("/App.tsx"),
                    "expected /App.tsx in final files"
                );
                assert!(
                    !post_process.files.contains_key("/src/main.tsx"),
                    "/src/main.tsx should not be emitted (Sandpack template owns entry)"
                );
                assert!(
                    post_process.files.contains_key("/styles.css"),
                    "expected /styles.css (template path) in final files"
                );
            }
            other => panic!("unexpected trailing events: {other:?}"),
        }
    }

    #[tokio::test]
    async fn streaming_events_arrive_incrementally_in_order() {
        let client = Arc::new(MockLlmClient::default());
        let runner = AgentRunner::with_mock_store(client, MockStore::new("mock"));

        let (tx, mut rx) = mpsc::channel::<AgentEvent>(1);

        let handle = tokio::spawn({
            let runner = std::sync::Arc::new(runner);
            let ctx = AgentContext {
                project_id: Some("project-1".to_string()),
                mock_config: Some(all_nodes_mock_config()),
                user_request: "build a todo app".to_string(),
            };
            async move {
                runner.run_streaming(ctx, tx).await;
            }
        });

        let first = timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("first event should arrive quickly")
            .expect("channel should not close before Analysis");
        assert!(matches!(first, AgentEvent::Analysis(_)));

        let mut remaining = vec![first];
        while let Some(evt) = rx.recv().await {
            remaining.push(evt);
        }

        let idx_analysis = remaining
            .iter()
            .position(|e| matches!(e, AgentEvent::Analysis(_)))
            .unwrap();
        let idx_intent = remaining
            .iter()
            .position(|e| matches!(e, AgentEvent::Intent(_)))
            .unwrap();
        let idx_post_process = remaining
            .iter()
            .position(|e| matches!(e, AgentEvent::PostProcess(_)))
            .unwrap();
        assert!(idx_analysis < idx_intent);
        assert!(idx_intent < idx_post_process);
        assert!(matches!(remaining.last(), Some(AgentEvent::Done)));

        handle.await.expect("producer task should finish");
    }

    #[tokio::test]
    async fn streaming_stops_when_receiver_dropped() {
        let client = Arc::new(MockLlmClient::default());
        let runner = AgentRunner::with_mock_store(client, MockStore::new("mock"));

        let (tx, rx) = mpsc::channel::<AgentEvent>(1);
        drop(rx);

        let ctx = AgentContext {
            project_id: Some("project-1".to_string()),
            mock_config: Some(all_nodes_mock_config()),
            user_request: "build a todo app".to_string(),
        };

        timeout(Duration::from_secs(3), runner.run_streaming(ctx, tx))
            .await
            .expect("run_streaming should return promptly when receiver is dropped");
    }

    fn mock_config_without_component_gen() -> serde_json::Value {
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
            "componentGenNode": false,
            "pageGenNode": true,
            "layoutNode": true,
            "styleGenNode": true,
            "appGenNode": true
        })
    }

    #[tokio::test]
    async fn streaming_emits_component_gen_partial_events_before_final() {
        let client = Arc::new(MockLlmClient::new(
            r#"{"path":"/components/Fake.tsx","content":"export default () => null;"}"#,
        ));
        let runner = AgentRunner::with_mock_store(client, MockStore::new("mock"));

        let events = collect(
            &runner,
            AgentContext {
                project_id: Some("project-1".to_string()),
                mock_config: Some(mock_config_without_component_gen()),
                user_request: "build a todo app".to_string(),
            },
        )
        .await;

        let partial_count = events
            .iter()
            .filter(|e| matches!(e, AgentEvent::ComponentGenPartial(_)))
            .count();
        assert!(
            partial_count >= 1,
            "expected at least one ComponentGenPartial event, got {partial_count} in {events:?}"
        );

        let idx_first_partial = events
            .iter()
            .position(|e| matches!(e, AgentEvent::ComponentGenPartial(_)))
            .expect("partial should exist");
        let idx_final = events
            .iter()
            .position(|e| matches!(e, AgentEvent::ComponentGen(_)))
            .expect("final componentGen should exist");
        assert!(
            idx_first_partial < idx_final,
            "partial events must precede final ComponentGen"
        );

        let progresses: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let AgentEvent::ComponentGenPartial(p) = e {
                    Some(p.progress)
                } else {
                    None
                }
            })
            .collect();
        let total = progresses[0].total;
        assert!(progresses.iter().all(|p| p.total == total));
        let max_completed = progresses.iter().map(|p| p.completed).max().unwrap();
        assert_eq!(max_completed, total);
    }
}
