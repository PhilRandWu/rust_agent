use crate::agent::flows::traditional::layout::LayoutInput;
use crate::llm::message::LlmMessage;

pub fn layout_messages(input: &LayoutInput) -> Vec<LlmMessage> {
    let ui_json = serde_json::to_string(&input.ui).unwrap_or_else(|_| "{}".to_string());
    let structure_json =
        serde_json::to_string(&input.structure).unwrap_or_else(|_| "{}".to_string());
    let pages_json = serde_json::to_string(&input.pages).unwrap_or_else(|_| "{}".to_string());

    vec![
        LlmMessage::system(
            r#"You are a senior React layout architect.

Your task is to generate reusable layout components for a React + TypeScript Sandpack project.

Return only valid JSON. Do not include markdown.

Output JSON shape:
{
  "layoutsCode": [
    {
      "path": "/layouts/MainLayout.tsx",
      "content": "import React from 'react';\nimport { Outlet } from 'react-router-dom';\n\nexport default function MainLayout() {\n  return <main><Outlet /></main>;\n}\n",
      "description": "Main layout",
      "pages": ["Home"]
    }
  ],
  "routeStructure": {
    "MainLayout": ["Home"]
  }
}

Rules:
1. Generate layout files only when multiple pages share structure.
2. Layout files should live in /layouts.
3. Every layout must export default a React component.
4. Prefer children prop for simple Sandpack apps.
5. Use Outlet only if routing is clearly planned.
6. Use Tailwind CSS classes.
7. Keep layouts simple.
8. Do not generate page code here.
9. Do not generate component business logic here.
10. For single page apps, returning empty layoutsCode is allowed."#,
        ),
        LlmMessage::user(format!(
            "UI JSON: {}\nStructure JSON: {}\nPages JSON: {}",
            ui_json, structure_json, pages_json
        )),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::layout::LayoutInput;
    use crate::agent::flows::traditional::page_gen::PageGenOutput;
    use crate::agent::flows::traditional::structure::StructureOutput;
    use crate::agent::flows::traditional::ui::{LayoutPattern, UiOutput, VisualStyle};
    use crate::llm::message::LlmRole;

    #[test]
    fn builds_layout_messages() {
        let messages = layout_messages(&LayoutInput::new(
            UiOutput {
                layout_pattern: LayoutPattern::SinglePage,
                visual_style: VisualStyle::Modern,
                primary_sections: vec!["Main".to_string()],
                interactions: Vec::new(),
                responsive_notes: Vec::new(),
            },
            StructureOutput { files: Vec::new() },
            PageGenOutput { files: Vec::new() },
        ));

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, LlmRole::System);
        assert!(messages[0].content.contains("layoutsCode"));
        assert!(messages[0].content.contains("routeStructure"));
        assert!(messages[1].content.contains("UI JSON"));
    }
}
