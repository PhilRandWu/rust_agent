use crate::agent::flows::traditional::page_gen::PageGenInput;
use crate::llm::message::LlmMessage;

pub fn page_gen_messages(input: &PageGenInput) -> Vec<LlmMessage> {
    let structure_json =
        serde_json::to_string(&input.structure).unwrap_or_else(|_| "{}".to_string());
    let ui_json = serde_json::to_string(&input.ui).unwrap_or_else(|_| "{}".to_string());
    let components_json =
        serde_json::to_string(&input.components).unwrap_or_else(|_| "{}".to_string());
    let component_code_json =
        serde_json::to_string(&input.component_code).unwrap_or_else(|_| "{}".to_string());
    let hooks_json = serde_json::to_string(&input.hooks).unwrap_or_else(|_| "{}".to_string());
    let types_json = serde_json::to_string(&input.types).unwrap_or_else(|_| "{}".to_string());

    vec![
        LlmMessage::system(
            r#"You are a senior React page engineer.

Your task is to generate React page files for a React + TypeScript Sandpack project.

Return only valid JSON. Do not include markdown.

Output JSON shape:
{
  "files": [
    {
      "path": "/pages/Home.tsx",
      "content": "import React from 'react';\n\nexport default function Home() {\n  return <main />;\n}\n",
      "description": "Home page"
    }
  ]
}

Rules:
1. Generate files only for planned /pages/*.tsx files.
2. Every page must export default a React component.
3. Pages should compose generated business components.
4. Pages may use hooks from /hooks.
5. Do not import mock data directly.
6. Do not duplicate business component implementations inside pages.
7. Use Tailwind CSS classes.
8. Use relative imports from the page file location.
9. Keep pages layout-focused.
10. Handle empty states gracefully.
11. Use TypeScript and TSX.
12. Do not include comments."#,
        ),
        LlmMessage::user(format!(
            "Structure JSON: {}\nUI JSON: {}\nComponent Specs JSON: {}\nComponent Code JSON: {}\nHooks JSON: {}\nTypes JSON: {}",
            structure_json, ui_json, components_json, component_code_json, hooks_json, types_json
        )),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::component::ComponentOutput;
    use crate::agent::flows::traditional::component_gen::ComponentGenOutput;
    use crate::agent::flows::traditional::hooks::HooksOutput;
    use crate::agent::flows::traditional::page_gen::PageGenInput;
    use crate::agent::flows::traditional::structure::StructureOutput;
    use crate::agent::flows::traditional::typegen::TypeOutput;
    use crate::agent::flows::traditional::ui::{LayoutPattern, UiOutput, VisualStyle};
    use crate::llm::message::LlmRole;

    #[test]
    fn builds_page_gen_messages() {
        let messages = page_gen_messages(&PageGenInput::new(
            StructureOutput { files: Vec::new() },
            UiOutput {
                layout_pattern: LayoutPattern::SinglePage,
                visual_style: VisualStyle::Modern,
                primary_sections: vec!["Hero".to_string()],
                interactions: vec!["Create todo".to_string()],
                responsive_notes: vec!["Stack on mobile".to_string()],
            },
            ComponentOutput {
                components: Vec::new(),
            },
            ComponentGenOutput { files: Vec::new() },
            HooksOutput { files: Vec::new() },
            TypeOutput { files: Vec::new() },
        ));

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, LlmRole::System);
        assert!(messages[0].content.contains("/pages/*.tsx"));
        assert!(messages[0].content.contains("export default"));
        assert!(messages[1].content.contains("UI JSON"));
    }
}
