use crate::agent::flows::traditional::capability::CapabilityOutput;
use crate::agent::flows::traditional::intent::IntentOutput;
use crate::agent::flows::traditional::ui::UiOutput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentInput {
    pub intent: IntentOutput,
    pub capabilities: CapabilityOutput,
    pub ui: UiOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentOutput {
    pub components: Vec<ComponentSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentSpec {
    pub component_id: String,
    pub original_id: String,
    pub component_type: String,
    pub description: String,
    pub props: Vec<ComponentProp>,
    pub events: Vec<ComponentEvent>,
    pub data_dependencies: Vec<String>,
    pub shadcn_component: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentProp {
    pub name: String,
    pub prop_type: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComponentEvent {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ComponentEventParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentEventParameter {
    pub name: String,
    pub parameter_type: String,
}

impl ComponentInput {
    pub fn new(intent: IntentOutput, capabilities: CapabilityOutput, ui: UiOutput) -> Self {
        Self {
            intent,
            capabilities,
            ui,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::capability::schema::CapabilityPriority;
    use crate::agent::flows::traditional::capability::{CapabilityItem, CapabilityOutput};
    use crate::agent::flows::traditional::intent::{IntentOutput, IntentType};
    use crate::agent::flows::traditional::ui::{LayoutPattern, UiOutput, VisualStyle};

    #[test]
    fn builds_component_input() {
        let input = ComponentInput::new(
            IntentOutput {
                intent_type: IntentType::Create,
                goal: "Create a todo app".to_string(),
                constraints: vec!["Keep UI simple".to_string()],
                user_stories: vec!["As a user, I can create todos".to_string()],
            },
            CapabilityOutput {
                capabilities: vec![CapabilityItem {
                    name: "Todo creation".to_string(),
                    description: "Users can create todo items".to_string(),
                    priority: CapabilityPriority::MustHave,
                }],
            },
            UiOutput {
                layout_pattern: LayoutPattern::SinglePage,
                visual_style: VisualStyle::Modern,
                primary_sections: vec!["Todo input".to_string(), "Todo list".to_string()],
                interactions: vec!["Create todo".to_string()],
                responsive_notes: vec!["Stack sections on mobile".to_string()],
            },
        );

        assert_eq!(input.intent.goal, "Create a todo app");
        assert_eq!(input.capabilities.capabilities.len(), 1);
        assert_eq!(input.ui.layout_pattern, LayoutPattern::SinglePage);
    }

    #[test]
    fn parses_component_output_from_camel_case_json() {
        let json = r#"
        {
          "components": [
            {
              "componentId": "TodoInputForm",
              "originalId": "Todo input",
              "componentType": "Form",
              "description": "Create new todo items",
              "props": [
                {
                  "name": "placeholder",
                  "propType": "string",
                  "description": "Input placeholder text",
                  "required": false
                }
              ],
              "events": [
                {
                  "name": "onCreateTodo",
                  "description": "Triggered when user submits a todo",
                  "parameters": [
                    {
                      "name": "title",
                      "parameterType": "string"
                    }
                  ]
                }
              ],
              "dataDependencies": ["Todo"],
              "shadcnComponent": "Form"
            }
          ]
        }
        "#;

        let output: ComponentOutput = serde_json::from_str(json).unwrap();

        assert_eq!(output.components.len(), 1);
        assert_eq!(output.components[0].component_id, "TodoInputForm");
        assert_eq!(output.components[0].component_type, "Form");
        assert_eq!(output.components[0].props[0].prop_type, "string");
        assert_eq!(
            output.components[0].events[0].parameters[0].parameter_type,
            "string"
        );
        assert_eq!(
            output.components[0].shadcn_component,
            Some("Form".to_string())
        );
    }
}
