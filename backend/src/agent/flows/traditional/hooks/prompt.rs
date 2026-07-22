use crate::agent::flows::traditional::hooks::HooksInput;
use crate::llm::message::LlmMessage;

pub fn hooks_messages(input: &HooksInput) -> Vec<LlmMessage> {
    let types_json = serde_json::to_string(&input.types).unwrap_or_else(|_| "{}".to_string());
    let service_json = serde_json::to_string(&input.service).unwrap_or_else(|_| "{}".to_string());
    let structure_json =
        serde_json::to_string(&input.structure).unwrap_or_else(|_| "{}".to_string());

    vec![
        LlmMessage::system(
            r#"You are a senior React hooks engineer.

Your task is to generate custom React hooks for a React + TypeScript Sandpack project.

Return only valid JSON. Do not include markdown.

Output JSON shape:
{
  "files": [
    {
      "path": "/hooks/useTodos.ts",
      "content": "import { useMemo, useState } from 'react';\nimport type { Todo } from '../types/todo';\nimport { getAllTodos } from '../services/todoService';\n\nexport function useTodos() {\n  const [todos, setTodos] = useState<Todo[]>(() => getAllTodos());\n  return { todos, setTodos };\n}\n",
      "description": "Todo state hook"
    }
  ]
}

Rules:
1. Generate one hook per domain model when possible.
2. Hook files should live in /hooks.
3. Hook names must start with use.
4. Use service functions as the data source.
5. Use React useState/useMemo when useful.
6. Do not fetch from real network APIs.
7. Do not import mock data directly when a service exists.
8. Use relative imports from hook file location.
9. Return plain objects from hooks.
10. Keep hooks simple and preview-friendly.
11. Use TypeScript only.
12. Do not include comments."#,
        ),
        LlmMessage::user(format!(
            "Types JSON: {}\nService JSON: {}\nStructure JSON: {}",
            types_json, service_json, structure_json
        )),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::hooks::HooksInput;
    use crate::agent::flows::traditional::service::{ServiceFile, ServiceOutput};
    use crate::agent::flows::traditional::structure::StructureOutput;
    use crate::agent::flows::traditional::typegen::{TypeFile, TypeOutput};
    use crate::llm::message::LlmRole;

    #[test]
    fn builds_hooks_messages_with_types_and_service() {
        let messages = hooks_messages(&HooksInput::new(
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
        ));

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, LlmRole::System);
        assert_eq!(messages[1].role, LlmRole::User);
        assert!(messages[0].content.contains("useTodos"));
        assert!(messages[1].content.contains("/services/todoService.ts"));
        assert!(messages[1].content.contains("/types/todo.ts"));
    }
}
