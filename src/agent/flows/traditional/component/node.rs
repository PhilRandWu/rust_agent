use crate::agent::flows::traditional::component::prompt::component_messages;
use crate::agent::flows::traditional::component::schema::{ComponentInput, ComponentOutput};
use crate::llm::client::LlmClient;
use crate::llm::structured::structured_chat;
use std::sync::Arc;

pub struct ComponentNode {
    client: Arc<dyn LlmClient>,
}

impl ComponentNode {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    pub async fn run(&self, input: ComponentInput) -> anyhow::Result<ComponentOutput> {
        let messages = component_messages(&input);
        structured_chat(self.client.as_ref(), &messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::capability::schema::CapabilityPriority;
    use crate::agent::flows::traditional::capability::{CapabilityItem, CapabilityOutput};
    use crate::agent::flows::traditional::intent::{IntentOutput, IntentType};
    use crate::agent::flows::traditional::ui::{LayoutPattern, UiOutput, VisualStyle};
    use crate::llm::mock::MockLlmClient;

    #[tokio::test]
    async fn parses_component_output_from_llm_json() {
        let client = Arc::new(MockLlmClient::new(
            r#"{
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
                },
                {
                  "componentId": "TodoListPanel",
                  "originalId": "Todo list",
                  "componentType": "List",
                  "description": "Render todo items",
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
                      "name": "onToggleTodo",
                      "description": "Triggered when user toggles todo completion",
                      "parameters": [
                        {
                          "name": "id",
                          "parameterType": "string"
                        }
                      ]
                    }
                  ],
                  "dataDependencies": ["Todo"],
                  "shadcnComponent": null
                }
              ]
            }"#,
        ));

        let node = ComponentNode::new(client);

        let output = node
            .run(ComponentInput::new(
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
            ))
            .await
            .expect("component node should parse output");

        assert_eq!(output.components.len(), 2);
        assert_eq!(output.components[0].component_id, "TodoInputForm");
        assert_eq!(output.components[0].props[0].prop_type, "string");
        assert_eq!(output.components[1].component_id, "TodoListPanel");
        assert_eq!(output.components[1].events[0].name, "onToggleTodo");
    }
}
