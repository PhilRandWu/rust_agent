use crate::agent::flows::traditional::typegen::prompt::type_messages;
use crate::agent::flows::traditional::typegen::{TypeInput, TypeOutput};
use crate::llm::client::LlmClient;
use crate::llm::structured::structured_chat;
use std::sync::Arc;

pub struct TypeNode {
    client: Arc<dyn LlmClient>,
}

impl TypeNode {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    #[tracing::instrument(skip_all, name = "node.typegen")]
    pub async fn run(&self, input: TypeInput) -> anyhow::Result<TypeOutput> {
        let messages = type_messages(&input);
        structured_chat(self.client.as_ref(), &messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::component::{
        ComponentEvent, ComponentEventParameter, ComponentOutput, ComponentProp, ComponentSpec,
    };
    use crate::agent::flows::traditional::structure::{
        FileKind, FileNode, GeneratedBy, StructureOutput,
    };
    use crate::llm::mock::MockLlmClient;

    #[tokio::test]
    async fn parses_type_output_from_llm_json() {
        let client = Arc::new(MockLlmClient::new(
            r#"{
              "files": [
                {
                  "path": "/types/todo.ts",
                  "code": "export interface Todo {\n  id: string;\n  title: string;\n  completed: boolean;\n}\n",
                  "modelId": "Todo"
                }
              ]
            }"#,
        ));

        let node = TypeNode::new(client);

        let output = node
            .run(TypeInput::new(
                ComponentOutput {
                    components: vec![ComponentSpec {
                        component_id: "TodoListPanel".to_string(),
                        original_id: "Todo list".to_string(),
                        component_type: "List".to_string(),
                        description: "Render todo items".to_string(),
                        props: vec![ComponentProp {
                            name: "todos".to_string(),
                            prop_type: "Todo[]".to_string(),
                            description: "Todo list data".to_string(),
                            required: true,
                        }],
                        events: vec![ComponentEvent {
                            name: "onToggleTodo".to_string(),
                            description: "Toggle todo completion".to_string(),
                            parameters: vec![ComponentEventParameter {
                                name: "id".to_string(),
                                parameter_type: "string".to_string(),
                            }],
                        }],
                        data_dependencies: vec!["Todo".to_string()],
                        shadcn_component: None,
                    }],
                },
                StructureOutput {
                    files: vec![FileNode {
                        path: "/types/todo.ts".to_string(),
                        kind: FileKind::New,
                        description: "Todo type definitions".to_string(),
                        source_correlation: Some("Todo".to_string()),
                        generated_by: GeneratedBy::TypeDefinition,
                    }],
                },
            ))
            .await
            .expect("type node should parse output");

        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].path, "/types/todo.ts");
        assert_eq!(output.files[0].model_id, "Todo");
        assert!(output.files[0].code.contains("interface Todo"));
    }
}
