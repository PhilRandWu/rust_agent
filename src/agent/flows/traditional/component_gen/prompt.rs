use crate::agent::flows::traditional::component::ComponentSpec;
use crate::agent::flows::traditional::component_gen::ComponentGenInput;
use crate::llm::message::LlmMessage;

pub fn single_component_messages(
    input: &ComponentGenInput,
    spec: &ComponentSpec,
) -> Vec<LlmMessage> {
    let spec_json = serde_json::to_string(spec).unwrap_or_else(|_| "{}".to_string());
    let types_json = serde_json::to_string(&input.types).unwrap_or_else(|_| "{}".to_string());
    let hooks_json = serde_json::to_string(&input.hooks).unwrap_or_else(|_| "{}".to_string());
    let service_json = serde_json::to_string(&input.service).unwrap_or_else(|_| "{}".to_string());

    vec![
        LlmMessage::system(
            r#"You are a senior React + TypeScript component engineer.

Generate a single React component file from the given specification.

Return only valid JSON matching this shape (no markdown fences):
{
  "path": "/components/ComponentName.tsx",
  "content": "...tsx source...",
  "description": "one-line description"
}

Rules:
1. Export default a React component.
2. Use TypeScript + TSX + Tailwind classes.
3. Import types from /types when needed (relative path).
4. Prefer hooks from /hooks.
5. Only generate ONE file for the specified component."#,
        ),
        LlmMessage::user(format!(
            "Component spec:\n{spec_json}\n\nAvailable types:\n{types_json}\n\nAvailable hooks:\n{hooks_json}\n\nAvailable services:\n{service_json}"
        )),
    ]
}

pub fn component_gen_messages(input: &ComponentGenInput) -> Vec<LlmMessage> {
    let structure_json =
        serde_json::to_string(&input.structure).unwrap_or_else(|_| "{}".to_string());
    let components_json =
        serde_json::to_string(&input.components).unwrap_or_else(|_| "{}".to_string());
    let types_json = serde_json::to_string(&input.types).unwrap_or_else(|_| "{}".to_string());
    let service_json = serde_json::to_string(&input.service).unwrap_or_else(|_| "{}".to_string());
    let hooks_json = serde_json::to_string(&input.hooks).unwrap_or_else(|_| "{}".to_string());

    vec![
        LlmMessage::system(
            r#"You are a senior React + TypeScript component engineer.

Your task is to generate real React component files from component specifications.

Return only valid JSON. Do not include markdown.

Output JSON shape:
{
  "files": [
    {
      "path": "/components/TodoListPanel.tsx",
      "content": "import React from 'react';\n\nexport default function TodoListPanel() {\n  return <div />;\n}\n",
      "description": "Todo list component"
    }
  ]
}

Rules:
1. Generate files only for planned /components/*.tsx files.
2. Every file must export default a React component.
3. Use TypeScript and TSX.
4. Use Tailwind CSS classes.
5. Prefer hooks from /hooks for data.
6. Use service functions only as fallback.
7. Import types from /types when needed.
8. Use relative imports from the component file location.
9. Do not import mock data directly.
10. Do not include comments.
11. Do not use browser-only APIs during render.
12. Keep components simple and preview-friendly.
13. Empty states should be handled gracefully.
14. Event props should be optional when possible.
15. Do not generate page files in this node."#,
        ),
        LlmMessage::user(format!(
            "Structure JSON: {}\nComponent Specs JSON: {}\nTypes JSON: {}\nService JSON: {}\nHooks JSON: {}",
            structure_json, components_json, types_json, service_json, hooks_json
        )),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::component::ComponentOutput;
    use crate::agent::flows::traditional::component_gen::ComponentGenInput;
    use crate::agent::flows::traditional::hooks::HooksOutput;
    use crate::agent::flows::traditional::service::ServiceOutput;
    use crate::agent::flows::traditional::structure::StructureOutput;
    use crate::agent::flows::traditional::typegen::TypeOutput;
    use crate::llm::message::LlmRole;

    #[test]
    fn builds_component_gen_messages() {
        let messages = component_gen_messages(&ComponentGenInput::new(
            StructureOutput { files: Vec::new() },
            ComponentOutput {
                components: Vec::new(),
            },
            TypeOutput { files: Vec::new() },
            ServiceOutput { files: Vec::new() },
            HooksOutput { files: Vec::new() },
        ));

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, LlmRole::System);
        assert_eq!(messages[1].role, LlmRole::User);
        assert!(messages[0].content.contains("/components/*.tsx"));
        assert!(messages[0].content.contains("export default"));
        assert!(messages[1].content.contains("Component Specs JSON"));
    }
}
