use crate::agent::flows::traditional::hooks::prompt::hooks_messages;
use crate::agent::flows::traditional::hooks::{HooksInput, HooksOutput};
use crate::llm::client::LlmClient;
use crate::llm::structured::structured_chat;
use std::sync::Arc;

pub struct HooksNode {
    client: Arc<dyn LlmClient>,
}

impl HooksNode {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    pub async fn run(&self, input: HooksInput) -> anyhow::Result<HooksOutput> {
        if input.service.files.is_empty() {
            return Ok(HooksOutput { files: Vec::new() });
        }

        let messages = hooks_messages(&input);
        structured_chat(self.client.as_ref(), &messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::service::{ServiceFile, ServiceOutput};
    use crate::agent::flows::traditional::structure::StructureOutput;
    use crate::agent::flows::traditional::typegen::{TypeFile, TypeOutput};
    use crate::llm::mock::MockLlmClient;

    #[tokio::test]
    async fn parses_hooks_output_from_llm_json() {
        let client = Arc::new(MockLlmClient::new(
            r#"{
              "files": [
                {
                  "path": "/hooks/useTodos.ts",
                  "content": "import { useState } from 'react';\nimport type { Todo } from '../types/todo';\nimport { getAllTodos } from '../services/todoService';\n\nexport function useTodos() {\n  const [todos, setTodos] = useState<Todo[]>(() => getAllTodos());\n  return { todos, setTodos };\n}\n",
                  "description": "Todo state hook"
                }
              ]
            }"#,
        ));

        let node = HooksNode::new(client);

        let output = node
            .run(HooksInput::new(
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
                StructureOutput { files: Vec::new() },
            ))
            .await
            .expect("hooks node should parse output");

        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].path, "/hooks/useTodos.ts");
        assert!(output.files[0].content.contains("useTodos"));
        assert!(output.files[0].content.contains("getAllTodos"));
    }

    #[tokio::test]
    async fn skips_when_no_service_exists() {
        let client = Arc::new(MockLlmClient::new("{}"));
        let node = HooksNode::new(client);

        let output = node
            .run(HooksInput::new(
                TypeOutput { files: Vec::new() },
                ServiceOutput { files: Vec::new() },
                StructureOutput { files: Vec::new() },
            ))
            .await
            .expect("hooks node should skip");

        assert!(output.files.is_empty());
    }
}
