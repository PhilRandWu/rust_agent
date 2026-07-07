use crate::agent::context::AgentContext;
use crate::agent::event::AgentEvent;
use crate::agent::flow_state::TraditionalFlowState;
use crate::agent::flows::traditional::analysis::{AnalysisInput, AnalysisNode};
use crate::agent::flows::traditional::app_gen::{AppGenInput, AppGenNode};
use crate::agent::flows::traditional::assemble::{AssembleInput, AssembleNode};
use crate::agent::flows::traditional::capability::{CapabilityInput, CapabilityNode};
use crate::agent::flows::traditional::component::{ComponentInput, ComponentNode};
use crate::agent::flows::traditional::component_gen::{ComponentGenInput, ComponentGenNode};
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
use std::sync::Arc;

pub struct AgentRunner {
    main_client: Arc<dyn LlmClient>,
    mock_store: MockStore,
}

impl AgentRunner {
    pub fn new(main_client: Arc<dyn LlmClient>) -> Self {
        Self {
            main_client,
            mock_store: MockStore::default(),
        }
    }

    pub fn with_mock_store(main_client: Arc<dyn LlmClient>, mock_store: MockStore) -> Self {
        Self {
            main_client,
            mock_store,
        }
    }

    pub async fn run(&self, context: AgentContext) -> Vec<AgentEvent> {
        let mut events = Vec::new();
        if let Err(error) = self.run_traditional(context, &mut events).await {
            events.push(AgentEvent::Error(error.to_string()));
        }
        events.push(AgentEvent::Done);
        events
    }

    async fn run_traditional(
        &self,
        context: AgentContext,
        events: &mut Vec<AgentEvent>,
    ) -> anyhow::Result<()> {
        let mut state = TraditionalFlowState::new();

        let analysis = if self.should_mock(&context, MockNode::Analysis) {
            self.mock_store.load(MockNode::Analysis).await?
        } else {
            let analysis_input = AnalysisInput::new(context.user_request.clone());
            let analysis_node = AnalysisNode::new(Arc::clone(&self.main_client));

            analysis_node.run(analysis_input).await?
        };

        events.push(AgentEvent::Analysis(analysis.clone()));
        state.analysis = Some(analysis.clone());

        let intent = if self.should_mock(&context, MockNode::Intent) {
            self.mock_store.load(MockNode::Intent).await?
        } else {
            let intent_input = IntentInput::new(analysis.clone(), context.user_request.clone());
            let intent_node = IntentNode::new(Arc::clone(&self.main_client));

            intent_node.run(intent_input).await?
        };

        events.push(AgentEvent::Intent(intent.clone()));
        state.intent = Some(intent.clone());

        let capability = if self.should_mock(&context, MockNode::Capability) {
            self.mock_store.load(MockNode::Capability).await?
        } else {
            let intent_input = CapabilityInput::new(analysis.clone(), intent.clone());
            let intent_node = CapabilityNode::new(Arc::clone(&self.main_client));

            intent_node.run(intent_input).await?
        };

        events.push(AgentEvent::Capability(capability.clone()));
        state.capability = Some(capability.clone());

        let ui = if self.should_mock(&context, MockNode::Ui) {
            self.mock_store.load(MockNode::Ui).await?
        } else {
            let ui_input = UiInput::new(analysis.clone(), intent.clone(), capability.clone());
            let ui_node = UiNode::new(Arc::clone(&self.main_client));

            ui_node.run(ui_input).await?
        };
        events.push(AgentEvent::Ui(ui.clone()));
        state.ui = Some(ui.clone());

        let component = if self.should_mock(&context, MockNode::Component) {
            self.mock_store.load(MockNode::Component).await?
        } else {
            let component_input =
                ComponentInput::new(intent.clone(), capability.clone(), ui.clone());
            let component_node = ComponentNode::new(Arc::clone(&self.main_client));

            component_node.run(component_input).await?
        };
        events.push(AgentEvent::Component(component.clone()));
        state.component = Some(component.clone());

        let structure = if self.should_mock(&context, MockNode::Structure) {
            self.mock_store.load(MockNode::Structure).await?
        } else {
            let structure_input =
                StructureInput::new(ui.clone(), component.clone(), capability.clone());
            let structure_node = StructureNode::new(Arc::clone(&self.main_client));

            structure_node.run(structure_input).await?
        };
        events.push(AgentEvent::Structure(structure.clone()));
        state.structure = Some(structure.clone());

        let dependency = if self.should_mock(&context, MockNode::Dependency) {
            self.mock_store.load(MockNode::Dependency).await?
        } else {
            DependencyNode::new()
                .run(DependencyInput::new(structure.clone()))
                .await?
        };
        events.push(AgentEvent::Dependency(dependency.clone()));
        state.dependency = Some(dependency.clone());

        let types = if self.should_mock(&context, MockNode::Type) {
            self.mock_store.load(MockNode::Type).await?
        } else {
            TypeNode::new(Arc::clone(&self.main_client))
                .run(TypeInput::new(component.clone(), structure.clone()))
                .await?
        };
        events.push(AgentEvent::Type(types.clone()));
        state.types = Some(types.clone());

        let utils = if self.should_mock(&context, MockNode::Utils) {
            self.mock_store.load(MockNode::Utils).await?
        } else {
            UtilsNode::new()
                .run(UtilsInput::new(structure.clone()))
                .await?
        };
        events.push(AgentEvent::Utils(utils.clone()));
        state.utils = Some(utils.clone());

        let mock_data = if self.should_mock(&context, MockNode::MockData) {
            self.mock_store.load(MockNode::MockData).await?
        } else {
            MockDataNode::new(Arc::clone(&self.main_client))
                .run(MockDataInput::new(types.clone(), structure.clone()))
                .await?
        };
        events.push(AgentEvent::MockData(mock_data.clone()));
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
        events.push(AgentEvent::Service(service.clone()));
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
        events.push(AgentEvent::Hooks(hooks.clone()));
        state.hooks = Some(hooks.clone());

        let component_gen = if self.should_mock(&context, MockNode::ComponentGen) {
            self.mock_store.load(MockNode::ComponentGen).await?
        } else {
            ComponentGenNode::new(Arc::clone(&self.main_client))
                .run(ComponentGenInput::new(
                    structure.clone(),
                    component.clone(),
                    types.clone(),
                    service.clone(),
                    hooks.clone(),
                ))
                .await?
        };
        events.push(AgentEvent::ComponentGen(component_gen.clone()));
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
        events.push(AgentEvent::PageGen(page_gen.clone()));
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
        events.push(AgentEvent::Layout(layout.clone()));
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
        events.push(AgentEvent::StyleGen(style_gen.clone()));
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
        events.push(AgentEvent::AppGen(app_gen.clone()));
        state.app_gen = Some(app_gen.clone());

        let assemble = if self.should_mock(&context, MockNode::Assemble) {
            self.mock_store.load(MockNode::Assemble).await?
        } else {
            AssembleNode::new()
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
                .await?
        };
        events.push(AgentEvent::Assemble(assemble.clone()));
        state.assemble = Some(assemble.clone());

        let post_process = if self.should_mock(&context, MockNode::PostProcess) {
            self.mock_store.load(MockNode::PostProcess).await?
        } else {
            PostProcessNode::new()
                .run(PostProcessInput::new(assemble))
                .await?
        };
        events.push(AgentEvent::PostProcess(post_process.clone()));
        state.post_process = Some(post_process);

        if let Some(files) = state.final_files() {
            events.push(AgentEvent::Files(files));
        }

        Ok(())
    }

    fn should_mock(&self, context: &AgentContext, node: MockNode) -> bool {
        MockStore::should_mock(context.mock_config.as_ref(), node)
    }
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
        let events = runner
            .run(AgentContext {
                project_id: Some("project-1".to_string()),
                mock_config: Some(json!({ "analysisNode": true })),
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
    async fn runs_full_traditional_pipeline_with_mock_store() {
        let client = Arc::new(MockLlmClient::default());
        let runner = AgentRunner::with_mock_store(client, MockStore::new("mock"));

        let events = runner
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
            [.., AgentEvent::Files(files), AgentEvent::Done] => {
                assert!(!files.files.is_empty(), "expected non-empty files");
            }
            other => panic!("unexpected trailing events: {other:?}"),
        }
    }
}
