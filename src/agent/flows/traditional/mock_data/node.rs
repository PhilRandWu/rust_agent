use crate::agent::flows::traditional::mock_data::prompt::mock_data_messages;
use crate::agent::flows::traditional::mock_data::{MockDataInput, MockDataOutput};
use crate::llm::client::LlmClient;
use crate::llm::structured::structured_chat;
use std::sync::Arc;

pub struct MockDataNode {
    client: Arc<dyn LlmClient>,
}

impl MockDataNode {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    pub async fn run(&self, input: MockDataInput) -> anyhow::Result<MockDataOutput> {
        if input.types.files.is_empty() {
            return Ok(MockDataOutput { files: Vec::new() });
        }

        let messages = mock_data_messages(&input);
        structured_chat(self.client.as_ref(), &messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::structure::{
        FileKind, FileNode, GeneratedBy, StructureOutput,
    };
    use crate::agent::flows::traditional::typegen::{TypeFile, TypeOutput};
    use crate::llm::mock::MockLlmClient;

    #[tokio::test]
    async fn parses_mock_data_output_from_llm_json() {
        let client = Arc::new(MockLlmClient::new(
            r#"{
              "files": [
                {
                  "path": "/data/todos.ts",
                  "content": "import type { Todo } from '../types/todo';\n\nexport const MOCK_TODOS: Todo[] = [\n  { id: 'todo-1', title: 'Learn Rust', completed: false },\n  { id: 'todo-2', title: 'Build agent', completed: true }\n];\n",
                  "description": "Todo mock data"
                }
              ]
            }"#,
        ));

        let node = MockDataNode::new(client);

        let output = node
            .run(MockDataInput::new(
                TypeOutput {
                    files: vec![TypeFile {
                        path: "/types/todo.ts".to_string(),
                        code: "export interface Todo { id: string; title: string; completed: boolean; }"
                            .to_string(),
                        model_id: "Todo".to_string(),
                    }],
                },
                StructureOutput {
                    files: vec![FileNode {
                        path: "/data/todos.ts".to_string(),
                        kind: FileKind::New,
                        description: "Todo mock data".to_string(),
                        source_correlation: Some("Todo".to_string()),
                        generated_by: GeneratedBy::MockData,
                    }],
                },
            ))
            .await
            .expect("mock data node should parse output");

        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].path, "/data/todos.ts");
        assert!(output.files[0].content.contains("MOCK_TODOS"));
        assert!(output.files[0].content.contains("Learn Rust"));
    }

    #[tokio::test]
    async fn returns_empty_files_when_no_types_exist() {
        let client = Arc::new(MockLlmClient::new("{}"));
        let node = MockDataNode::new(client);

        let output = node
            .run(MockDataInput::new(
                TypeOutput { files: Vec::new() },
                StructureOutput { files: Vec::new() },
            ))
            .await
            .expect("mock data node should skip empty types");

        assert!(output.files.is_empty());
    }
}
