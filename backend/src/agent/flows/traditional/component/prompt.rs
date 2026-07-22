use crate::agent::flows::traditional::component::schema::ComponentInput;
use crate::llm::message::LlmMessage;

pub fn component_messages(input: &ComponentInput) -> Vec<LlmMessage> {
    let intent_json = serde_json::to_string(&input.intent).unwrap_or_else(|_| "{}".to_string());

    let capability_json =
        serde_json::to_string(&input.capabilities).unwrap_or_else(|_| "{}".to_string());

    let ui_json = serde_json::to_string(&input.ui).unwrap_or_else(|_| "{}".to_string());

    vec![
        LlmMessage::system(
            r#"You are a senior frontend architect.

Your task is to convert UI planning, product intent, and capabilities into business component specifications.

Return only valid JSON. Do not include markdown.

Output JSON shape:
{
  "components": [
    {
      "componentId": "TodoInputForm",
      "originalId": "Todo input",
      "componentType": "Form",
      "description": "Create new todo items",
      "props": [
        {
          "name": "todos",
          "propType": "Todo[]",
          "description": "Todo list data",
          "required": true
        }
      ],
      "events": [
        {
          "name": "onCreateTodo",
          "description": "Triggered when user creates a todo",
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

Rules:
1. componentId must be PascalCase and business-specific.
2. Do not use generic names like Table, Form, Button, Card, List, Panel.
3. Prefer names like TodoListPanel, TodoInputForm, UserProfileCard.
4. prop names must be camelCase.
5. event names must start with on, such as onCreateTodo or onSelectTodo.
6. propType and parameterType must be TypeScript type strings.
7. dataDependencies should list business entity names used by the component.
8. shadcnComponent can be null if there is no obvious mapping.
9. Only include business components, not atomic UI components."#,
        ),
        LlmMessage::user(format!(
            "Intent JSON: {}\nCapability JSON: {}\nUI JSON: {}",
            intent_json, capability_json, ui_json
        )),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::capability::schema::CapabilityPriority;
    use crate::agent::flows::traditional::capability::{CapabilityItem, CapabilityOutput};
    use crate::agent::flows::traditional::intent::{IntentOutput, IntentType};
    use crate::agent::flows::traditional::ui::{LayoutPattern, UiOutput, VisualStyle};
    use crate::llm::message::LlmRole;

    #[test]
    fn builds_component_messages_with_intent_capability_and_ui() {
        let messages = component_messages(&ComponentInput::new(
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
        ));

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, LlmRole::System);
        assert_eq!(messages[1].role, LlmRole::User);
        assert!(messages[0].content.contains("componentId"));
        assert!(messages[0].content.contains("business-specific"));
        assert!(
            messages[1]
                .content
                .contains(r#""goal":"Create a todo app""#)
        );
        assert!(messages[1].content.contains(r#""name":"Todo creation""#));
        assert!(
            messages[1]
                .content
                .contains(r#""primary_sections":["Todo input","Todo list"]"#)
        );
    }
}
