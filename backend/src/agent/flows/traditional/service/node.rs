use crate::agent::flows::traditional::service::prompt::service_messages;
use crate::agent::flows::traditional::service::{ServiceInput, ServiceOutput};
use crate::llm::client::LlmClient;
use crate::llm::structured::structured_chat;
use std::sync::Arc;

pub struct ServiceNode {
    client: Arc<dyn LlmClient>,
}

impl ServiceNode {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    #[tracing::instrument(skip_all, name = "node.service")]
    pub async fn run(&self, input: ServiceInput) -> anyhow::Result<ServiceOutput> {
        if input.mock_data.files.is_empty() {
            return Ok(ServiceOutput { files: Vec::new() });
        }

        let messages = service_messages(&input);
        structured_chat(self.client.as_ref(), &messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::mock_data::{MockDataFile, MockDataOutput};
    use crate::agent::flows::traditional::structure::StructureOutput;
    use crate::agent::flows::traditional::typegen::{TypeFile, TypeOutput};
    use crate::agent::flows::traditional::utils::UtilsOutput;
    use crate::llm::mock::MockLlmClient;

    #[tokio::test]
    async fn parses_service_output_from_llm_json() {
        let client = Arc::new(MockLlmClient::new(
            r#"{
              "files": [
                {
                  "path": "/services/todoService.ts",
                  "content": "import type { Todo } from '../types/todo';\nimport { MOCK_TODOS } from '../data/todos';\n\nexport function getAllTodos(): Todo[] {\n  return MOCK_TODOS;\n}\n\nexport function getTodoById(id: string): Todo | undefined {\n  return MOCK_TODOS.find(todo => todo.id === id);\n}\n",
                  "description": "Todo data access service"
                }
              ]
            }"#,
        ));

        let node = ServiceNode::new(client);

        let output = node
            .run(ServiceInput::new(
                TypeOutput {
                    files: vec![TypeFile {
                        path: "/types/todo.ts".to_string(),
                        code: "export interface Todo { id: string; title: string; }".to_string(),
                        model_id: "Todo".to_string(),
                    }],
                },
                MockDataOutput {
                    files: vec![MockDataFile {
                        path: "/data/todos.ts".to_string(),
                        content: "export const MOCK_TODOS = [];".to_string(),
                        description: "Todo mock data".to_string(),
                    }],
                },
                UtilsOutput { files: Vec::new() },
                StructureOutput { files: Vec::new() },
            ))
            .await
            .expect("service node should parse output");

        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].path, "/services/todoService.ts");
        assert!(output.files[0].content.contains("getAllTodos"));
        assert!(output.files[0].content.contains("getTodoById"));
    }

    #[tokio::test]
    async fn skips_when_no_mock_data_exists() {
        let client = Arc::new(MockLlmClient::new("{}"));
        let node = ServiceNode::new(client);

        let output = node
            .run(ServiceInput::new(
                TypeOutput { files: Vec::new() },
                MockDataOutput { files: Vec::new() },
                UtilsOutput { files: Vec::new() },
                StructureOutput { files: Vec::new() },
            ))
            .await
            .expect("service node should skip");

        assert!(output.files.is_empty());
    }
}
