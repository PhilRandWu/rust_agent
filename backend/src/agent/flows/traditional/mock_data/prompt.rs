use crate::agent::flows::traditional::mock_data::MockDataInput;
use crate::llm::message::LlmMessage;

pub fn mock_data_messages(input: &MockDataInput) -> Vec<LlmMessage> {
    let types_json = serde_json::to_string(&input.types).unwrap_or_else(|_| "{}".to_string());

    let structure_json =
        serde_json::to_string(&input.structure).unwrap_or_else(|_| "{}".to_string());

    vec![
        LlmMessage::system(
            r#"You are a TypeScript mock data generator.

Your task is to generate realistic mock data files for a React + TypeScript Sandpack project.

Return only valid JSON. Do not include markdown.

Output JSON shape:
{
  "files": [
    {
      "path": "/data/todos.ts",
      "content": "import type { Todo } from '../types/todo';\n\nexport const MOCK_TODOS: Todo[] = [...];\n",
      "description": "Todo mock data"
    }
  ]
}

Rules:
1. Generate mock data files for every type model.
2. Prefer planned /data/*.ts paths from structure when available.
3. If no planned data path exists, use /data/{lowercaseModel}s.ts.
4. Import types from ../types/{file}.
5. Export constants with uppercase names, such as MOCK_TODOS.
6. Include 3 to 5 realistic mock records.
7. Do not include helper functions.
8. Do not include React code.
9. Use TypeScript only.
10. Keep content valid and directly usable."#,
        ),
        LlmMessage::user(format!(
            "Type JSON: {}\nStructure JSON: {}",
            types_json, structure_json
        )),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::mock_data::MockDataInput;
    use crate::agent::flows::traditional::structure::{
        FileKind, FileNode, GeneratedBy, StructureOutput,
    };
    use crate::agent::flows::traditional::typegen::{TypeFile, TypeOutput};
    use crate::llm::message::LlmRole;

    #[test]
    fn builds_mock_data_messages_with_types_and_structure() {
        let messages = mock_data_messages(&MockDataInput::new(
            TypeOutput {
                files: vec![TypeFile {
                    path: "/types/todo.ts".to_string(),
                    code: "export interface Todo { id: string; title: string; }".to_string(),
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
        ));

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, LlmRole::System);
        assert_eq!(messages[1].role, LlmRole::User);
        assert!(messages[0].content.contains("MOCK_TODOS"));
        assert!(messages[1].content.contains("/types/todo.ts"));
        assert!(messages[1].content.contains("/data/todos.ts"));
    }
}
