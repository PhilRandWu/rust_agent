use crate::agent::flows::traditional::structure::prompt::structure_messages;
use crate::agent::flows::traditional::structure::{StructureInput, StructureOutput};
use crate::llm::client::LlmClient;
use crate::llm::structured::structured_chat;
use std::sync::Arc;

pub struct StructureNode {
    client: Arc<dyn LlmClient>,
}

impl StructureNode {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    #[tracing::instrument(skip_all, name = "node.structure")]
    pub async fn run(&self, input: StructureInput) -> anyhow::Result<StructureOutput> {
        let messages = structure_messages(&input);
        structured_chat(self.client.as_ref(), &messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::capability::{
        CapabilityItem, CapabilityOutput, CapabilityPriority,
    };
    use crate::agent::flows::traditional::component::{
        ComponentEvent, ComponentEventParameter, ComponentOutput, ComponentProp, ComponentSpec,
    };
    use crate::agent::flows::traditional::structure::{FileKind, GeneratedBy, StructureInput};
    use crate::agent::flows::traditional::ui::{LayoutPattern, UiOutput, VisualStyle};
    use crate::llm::mock::MockLlmClient;

    #[tokio::test]
    async fn parses_structure_output_from_llm_json() {
        let client = Arc::new(MockLlmClient::new(
            r#"{
              "files": [
                {
                  "path": "/App.tsx",
                  "kind": "overwrite",
                  "description": "Application entry component",
                  "sourceCorrelation": null,
                  "generatedBy": "appEntry"
                },
                {
                  "path": "/pages/Home.tsx",
                  "kind": "new",
                  "description": "Home page",
                  "sourceCorrelation": "Home",
                  "generatedBy": "page"
                },
                {
                  "path": "/components/TodoInputForm.tsx",
                  "kind": "new",
                  "description": "Todo input component",
                  "sourceCorrelation": "TodoInputForm",
                  "generatedBy": "component"
                },
                {
                  "path": "/styles.css",
                  "kind": "template",
                  "description": "Global styles",
                  "sourceCorrelation": null,
                  "generatedBy": "template"
                }
              ]
            }"#,
        ));

        let node = StructureNode::new(client);

        let output = node
            .run(StructureInput::new(
                UiOutput {
                    layout_pattern: LayoutPattern::SinglePage,
                    visual_style: VisualStyle::Modern,
                    primary_sections: vec!["Todo input".to_string(), "Todo list".to_string()],
                    interactions: vec!["Create todo".to_string()],
                    responsive_notes: vec!["Stack content on mobile".to_string()],
                },
                ComponentOutput {
                    components: vec![ComponentSpec {
                        component_id: "TodoInputForm".to_string(),
                        original_id: "Todo input".to_string(),
                        component_type: "Form".to_string(),
                        description: "Create todos".to_string(),
                        props: vec![ComponentProp {
                            name: "placeholder".to_string(),
                            prop_type: "string".to_string(),
                            description: "Input placeholder".to_string(),
                            required: false,
                        }],
                        events: vec![ComponentEvent {
                            name: "onCreateTodo".to_string(),
                            description: "Create todo".to_string(),
                            parameters: vec![ComponentEventParameter {
                                name: "title".to_string(),
                                parameter_type: "string".to_string(),
                            }],
                        }],
                        data_dependencies: vec!["Todo".to_string()],
                        shadcn_component: Some("Form".to_string()),
                    }],
                },
                CapabilityOutput {
                    capabilities: vec![CapabilityItem {
                        name: "Todo creation".to_string(),
                        description: "Users can create todos".to_string(),
                        priority: CapabilityPriority::MustHave,
                    }],
                },
            ))
            .await
            .expect("structure node should parse output");

        assert_eq!(output.files.len(), 4);
        assert_eq!(output.files[0].path, "/App.tsx");
        assert_eq!(output.files[0].kind, FileKind::Overwrite);
        assert_eq!(output.files[0].generated_by, GeneratedBy::AppEntry);
        assert_eq!(
            output.files[2].source_correlation,
            Some("TodoInputForm".to_string())
        );
    }
}
