use crate::agent::flows::traditional::structure::schema::StructureInput;
use crate::llm::message::LlmMessage;

pub fn structure_messages(input: &StructureInput) -> Vec<LlmMessage> {
    let ui_json = serde_json::to_string(&input.ui).unwrap_or_else(|_| "{}".to_string());

    let components_json =
        serde_json::to_string(&input.components).unwrap_or_else(|_| "{}".to_string());

    let capabilities_json =
        serde_json::to_string(&input.capabilities).unwrap_or_else(|_| "{}".to_string());

    vec![
        LlmMessage::system(
            r#"You are a senior frontend architecture planner.

Your task is to convert UI planning, component specifications, and capabilities into a concrete React + TypeScript project file structure for Sandpack.

Return only valid JSON. Do not include markdown.

Output JSON shape:
{
  "files": [
    {
      "path": "/App.tsx",
      "kind": "overwrite",
      "description": "Application entry component",
      "sourceCorrelation": null,
      "generatedBy": "appEntry"
    }
  ]
}

Rules:
1. path must start with "/".
2. Do not output package.json unless it must be changed.
3. Do not output tsconfig.json unless it must be changed.
4. Do not output node_modules or public/index.html.
5. Always include /App.tsx as kind overwrite and generatedBy appEntry.
6. Always include /styles.css as kind template and generatedBy template.
7. Generate one /components/{ComponentId}.tsx file for each business component.
8. Generate at least one /pages/Home.tsx file for a single page app.
9. Generate /lib/utils.ts as scaffold when shadcnComponent is used.
10. If a component has dataDependencies, generate /types/{entity}.ts and /data/{entity}.ts.
11. kind must be one of: template, overwrite, new.
12. generatedBy must be one of:
template, scaffold, typeDefinition, mockData, appEntry, component, hook, page."#,
        ),
        LlmMessage::user(format!(
            "UI JSON: {}\nComponent JSON: {}\nCapability JSON: {}",
            ui_json, components_json, capabilities_json
        )),
    ]
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
    use crate::llm::message::LlmRole;

    #[test]
    fn builds_structure_messages_with_ui_components_and_capabilities() {
        let messages = structure_messages(&StructureInput::new(
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
        ));

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, LlmRole::System);
        assert_eq!(messages[1].role, LlmRole::User);
        assert!(messages[0].content.contains("/App.tsx"));
        assert!(messages[0].content.contains("generatedBy"));
        assert!(
            messages[1]
                .content
                .contains(r#""componentId":"TodoInputForm""#)
        );
        assert!(
            messages[1]
                .content
                .contains(r#""layout_pattern":"SINGLE_PAGE""#)
        );
        assert!(messages[1].content.contains(r#""name":"Todo creation""#));
    }
}
