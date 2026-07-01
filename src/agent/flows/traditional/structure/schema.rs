use crate::agent::flows::traditional::capability::CapabilityOutput;
use crate::agent::flows::traditional::component::ComponentOutput;
use crate::agent::flows::traditional::ui::UiOutput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureInput {
    pub ui: UiOutput,
    pub components: ComponentOutput,
    pub capabilities: CapabilityOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructureOutput {
    pub files: Vec<FileNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileNode {
    pub path: String,
    pub kind: FileKind,
    pub description: String,
    pub source_correlation: Option<String>,
    pub generated_by: GeneratedBy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum FileKind {
    Template,
    Overwrite,
    New,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum GeneratedBy {
    Template,
    Scaffold,
    TypeDefinition,
    MockData,
    AppEntry,
    Component,
    Hook,
    Page,
}

impl StructureInput {
    pub fn new(ui: UiOutput, components: ComponentOutput, capabilities: CapabilityOutput) -> Self {
        Self {
            ui,
            components,
            capabilities,
        }
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
    use crate::agent::flows::traditional::ui::{LayoutPattern, UiOutput, VisualStyle};

    #[test]
    fn builds_structure_input() {
        let input = StructureInput::new(
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
        );

        assert_eq!(input.components.components.len(), 1);
        assert_eq!(input.ui.layout_pattern, LayoutPattern::SinglePage);
        assert_eq!(input.capabilities.capabilities[0].name, "Todo creation");
    }

    #[test]
    fn parses_structure_output_from_camel_case_json() {
        let json = r#"
        {
          "files": [
            {
              "path": "/App.tsx",
              "kind": "overwrite",
              "description": "Application entry component",
              "sourceCorrelation": null,
              "generatedBy": "appEntry"
            },
            {
              "path": "/components/TodoInputForm.tsx",
              "kind": "new",
              "description": "Todo input business component",
              "sourceCorrelation": "TodoInputForm",
              "generatedBy": "component"
            }
          ]
        }
        "#;

        let output: StructureOutput = serde_json::from_str(json).unwrap();

        assert_eq!(output.files.len(), 2);
        assert_eq!(output.files[0].kind, FileKind::Overwrite);
        assert_eq!(output.files[0].generated_by, GeneratedBy::AppEntry);
        assert_eq!(
            output.files[1].source_correlation,
            Some("TodoInputForm".to_string())
        );
    }
}
