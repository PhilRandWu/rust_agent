use crate::agent::flows::traditional::service::ServiceInput;
use crate::llm::message::LlmMessage;

pub fn service_messages(input: &ServiceInput) -> Vec<LlmMessage> {
    let types_json = serde_json::to_string(&input.types).unwrap_or_else(|_| "{}".to_string());
    let mock_data_json =
        serde_json::to_string(&input.mock_data).unwrap_or_else(|_| "{}".to_string());
    let utils_json = serde_json::to_string(&input.utils).unwrap_or_else(|_| "{}".to_string());
    let structure_json =
        serde_json::to_string(&input.structure).unwrap_or_else(|_| "{}".to_string());

    vec![
        LlmMessage::system(
            r#"You are a senior TypeScript service layer engineer.

Your task is to generate service files for a React + TypeScript Sandpack project.

Return only valid JSON. Do not include markdown.

Output JSON shape:
{
  "files": [
    {
      "path": "/services/todoService.ts",
      "content": "import type { Todo } from '../types/todo';\nimport { MOCK_TODOS } from '../data/todos';\n\nexport function getAllTodos(): Todo[] {\n  return MOCK_TODOS;\n}\n",
      "description": "Todo data access service"
    }
  ]
}

Rules:
1. Generate one service file per domain model when possible.
2. Service files should live in /services.
3. Use mock data files as the data source.
4. Import types from /types files.
5. Import mock constants from /data files.
6. Use relative imports from the service file location.
7. Generate simple read functions only.
8. Always include getAllXxx().
9. Include getXxxById(id: string) when the model has an id field.
10. Do not generate create/update/delete unless strongly required.
11. Do not include React code.
12. Do not include comments.
13. Every function must have an explicit return type.
14. Use TypeScript only."#,
        ),
        LlmMessage::user(format!(
            "Types JSON: {}\nMock Data JSON: {}\nUtils JSON: {}\nStructure JSON: {}",
            types_json, mock_data_json, utils_json, structure_json
        )),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::mock_data::{MockDataFile, MockDataOutput};
    use crate::agent::flows::traditional::service::ServiceInput;
    use crate::agent::flows::traditional::structure::StructureOutput;
    use crate::agent::flows::traditional::typegen::{TypeFile, TypeOutput};
    use crate::agent::flows::traditional::utils::UtilsOutput;
    use crate::llm::message::LlmRole;

    #[test]
    fn builds_service_messages_with_previous_outputs() {
        let messages = service_messages(&ServiceInput::new(
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
        ));

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, LlmRole::System);
        assert_eq!(messages[1].role, LlmRole::User);
        assert!(messages[0].content.contains("getAllXxx"));
        assert!(messages[1].content.contains("/types/todo.ts"));
        assert!(messages[1].content.contains("/data/todos.ts"));
    }
}
