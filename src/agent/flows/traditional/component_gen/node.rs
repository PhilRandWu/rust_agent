use crate::agent::flows::traditional::component_gen::parallel::ComponentGenStream;
use crate::agent::flows::traditional::component_gen::{
    ComponentGenInput, ComponentGenOutput, ComponentGenPartial,
};
use crate::agent::flows::traditional::structure::GeneratedBy;
use crate::llm::client::LlmClient;
use crate::llm::semaphore::LlmGate;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct ComponentGenNode {
    client: Arc<dyn LlmClient>,
    gate: LlmGate,
}

impl ComponentGenNode {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self {
            client,
            gate: LlmGate::new(1),
        }
    }

    pub fn with_gate(client: Arc<dyn LlmClient>, gate: LlmGate) -> Self {
        Self { client, gate }
    }

    pub fn stream(&self, input: ComponentGenInput) -> Option<ComponentGenStream> {
        if !hash_component_files(&input) {
            return None;
        }
        Some(ComponentGenStream::spawn(
            Arc::clone(&self.client),
            self.gate.clone(),
            input,
        ))
    }

    #[tracing::instrument(skip_all, name = "node.component_gen")]
    pub async fn run(&self, input: ComponentGenInput) -> anyhow::Result<ComponentGenOutput> {
        self.run_with_partial(input, None).await
    }

    pub async fn run_with_partial(
        &self,
        input: ComponentGenInput,
        partial_tx: Option<mpsc::Sender<ComponentGenPartial>>,
    ) -> anyhow::Result<ComponentGenOutput> {
        let Some(mut stream) = self.stream(input) else {
            return Ok(ComponentGenOutput { files: Vec::new() });
        };
        while let Some(evt) = stream.next_partial().await {
            if let Some(tx) = &partial_tx {
                let _ = tx.send(evt).await;
            }
        }
        Ok(ComponentGenOutput {
            files: stream.finish()?,
        })
    }
}

fn hash_component_files(input: &ComponentGenInput) -> bool {
    input.structure.files.iter().any(|file| {
        file.generated_by == GeneratedBy::Component
            && file.path.starts_with("/components/")
            && file.path.ends_with(".tsx")
    })
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
              "path": "/components/TodoListPanel.tsx",
              "content": "import React from 'react';\nimport type { Todo } from '../types/todo';\n\nexport default function TodoListPanel() { return <div />; }\n",
              "description": "Todo list component"
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
