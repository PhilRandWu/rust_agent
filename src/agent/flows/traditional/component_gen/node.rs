use crate::agent::flows::traditional::component_gen::prompt::component_gen_messages;
use crate::agent::flows::traditional::component_gen::{ComponentGenInput, ComponentGenOutput};
use crate::agent::flows::traditional::structure::GeneratedBy;
use crate::llm::client::LlmClient;
use crate::llm::structured::structured_chat;
use std::sync::Arc;

pub struct ComponentGenNode {
    client: Arc<dyn LlmClient>,
}

impl ComponentGenNode {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    pub async fn run(&self, input: ComponentGenInput) -> anyhow::Result<ComponentGenOutput> {
        let has_component_files = input.structure.files.iter().any(|file| {
            file.generated_by == GeneratedBy::Component
                && file.path.starts_with("/components/")
                && file.path.ends_with(".tsx")
        });

        if !has_component_files {
            return Ok(ComponentGenOutput { files: Vec::new() });
        }

        let messages = component_gen_messages(&input);
        structured_chat(self.client.as_ref(), &messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::component::{
        ComponentEvent, ComponentEventParameter, ComponentOutput, ComponentProp, ComponentSpec,
    };
    use crate::agent::flows::traditional::hooks::{HooksFile, HooksOutput};
    use crate::agent::flows::traditional::service::{ServiceFile, ServiceOutput};
    use crate::agent::flows::traditional::structure::{
        FileKind, FileNode, GeneratedBy, StructureOutput,
    };
    use crate::agent::flows::traditional::typegen::{TypeFile, TypeOutput};
    use crate::llm::mock::MockLlmClient;

    #[tokio::test]
    async fn parses_component_gen_output_from_llm_json() {
        let client = Arc::new(MockLlmClient::new(
            r#"{
              "files": [
                {
                  "path": "/components/TodoListPanel.tsx",
                  "content": "import React from 'react';\nimport type { Todo } from '../types/todo';\n\ninterface TodoListPanelProps {\n  todos?: Todo[];\n}\n\nexport default function TodoListPanel({ todos = [] }: TodoListPanelProps) {\n  if (!todos.length) {\n    return <div className=\"rounded-xl border border-dashed border-slate-300 p-8 text-center text-slate-500\">No todos yet</div>;\n  }\n\n  return <div className=\"space-y-2\">{todos.map(todo => <div key={todo.id}>{todo.title}</div>)}</div>;\n}\n",
                  "description": "Todo list component"
                }
              ]
            }"#,
        ));

        let node = ComponentGenNode::new(client);

        let output = node
            .run(ComponentGenInput::new(
                StructureOutput {
                    files: vec![FileNode {
                        path: "/components/TodoListPanel.tsx".to_string(),
                        kind: FileKind::New,
                        description: "Todo list component".to_string(),
                        source_correlation: Some("TodoListPanel".to_string()),
                        generated_by: GeneratedBy::Component,
                    }],
                },
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
                            required: false,
                        }],
                        events: vec![ComponentEvent {
                            name: "onToggleTodo".to_string(),
                            description: "Toggle todo".to_string(),
                            parameters: vec![ComponentEventParameter {
                                name: "id".to_string(),
                                parameter_type: "string".to_string(),
                            }],
                        }],
                        data_dependencies: vec!["Todo".to_string()],
                        shadcn_component: None,
                    }],
                },
                TypeOutput {
                    files: vec![TypeFile {
                        path: "/types/todo.ts".to_string(),
                        code: "export interface Todo { id: string; title: string; }".to_string(),
                        model_id: "Todo".to_string(),
                    }],
                },
                ServiceOutput {
                    files: vec![ServiceFile {
                        path: "/services/todoService.ts".to_string(),
                        content: "export function getAllTodos() { return []; }".to_string(),
                        description: "Todo service".to_string(),
                    }],
                },
                HooksOutput {
                    files: vec![HooksFile {
                        path: "/hooks/useTodos.ts".to_string(),
                        content: "export function useTodos() { return { todos: [] }; }".to_string(),
                        description: "Todo hook".to_string(),
                    }],
                },
            ))
            .await
            .expect("component gen node should parse output");

        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].path, "/components/TodoListPanel.tsx");
        assert!(
            output.files[0]
                .content
                .contains("export default function TodoListPanel")
        );
    }

    #[tokio::test]
    async fn skips_when_no_component_files_exist() {
        let client = Arc::new(MockLlmClient::new("{}"));
        let node = ComponentGenNode::new(client);

        let output = node
            .run(ComponentGenInput::new(
                StructureOutput { files: Vec::new() },
                ComponentOutput {
                    components: Vec::new(),
                },
                TypeOutput { files: Vec::new() },
                ServiceOutput { files: Vec::new() },
                HooksOutput { files: Vec::new() },
            ))
            .await
            .expect("component gen node should skip");

        assert!(output.files.is_empty());
    }
}
