use crate::agent::flows::traditional::app_gen::AppGenInput;
use crate::llm::message::LlmMessage;

pub fn app_gen_messages(input: &AppGenInput) -> Vec<LlmMessage> {
    let ui_json = serde_json::to_string(&input.ui).unwrap_or_else(|_| "{}".to_string());
    let structure_json =
        serde_json::to_string(&input.structure).unwrap_or_else(|_| "{}".to_string());
    let pages_json = serde_json::to_string(&input.pages).unwrap_or_else(|_| "{}".to_string());
    let layouts_json = serde_json::to_string(&input.layouts).unwrap_or_else(|_| "{}".to_string());

    vec![
        LlmMessage::system(
            r#"You are a senior React application entry engineer.

Your task is to generate /App.tsx for a React + TypeScript Sandpack project.

Return only valid JSON. Do not include markdown.

Output JSON shape:
{
  "path": "/App.tsx",
  "content": "import React from 'react';\nimport Home from './pages/Home';\n\nexport default function App() {\n  return <Home />;\n}\n",
  "description": "Application root"
}

Rules:
1. path must be /App.tsx.
2. Generate only App.tsx.
3. Use generated pages from /pages.
4. Use generated layouts from /layouts only if they exist.
5. For single-page apps, render the main page directly.
6. Use relative imports from App.tsx.
7. Do not include mock data directly.
8. Do not duplicate page or component code.
9. Export default function App.
10. Use TypeScript and TSX.
11. Do not include comments."#,
        ),
        LlmMessage::user(format!(
            "UI JSON: {}\nStructure JSON: {}\nPages JSON: {}\nLayouts JSON: {}",
            ui_json, structure_json, pages_json, layouts_json
        )),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::app_gen::AppGenInput;
    use crate::agent::flows::traditional::layout::LayoutOutput;
    use crate::agent::flows::traditional::page_gen::PageGenOutput;
    use crate::agent::flows::traditional::structure::StructureOutput;
    use crate::agent::flows::traditional::ui::{LayoutPattern, UiOutput, VisualStyle};
    use crate::llm::message::LlmRole;
    use std::collections::BTreeMap;

    #[test]
    fn builds_app_gen_messages() {
        let messages = app_gen_messages(&AppGenInput::new(
            UiOutput {
                layout_pattern: LayoutPattern::SinglePage,
                visual_style: VisualStyle::Modern,
                primary_sections: vec!["Main".to_string()],
                interactions: Vec::new(),
                responsive_notes: Vec::new(),
            },
            StructureOutput { files: Vec::new() },
            PageGenOutput { files: Vec::new() },
            LayoutOutput {
                layouts_code: Vec::new(),
                route_structure: BTreeMap::new(),
            },
        ));

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, LlmRole::System);
        assert!(messages[0].content.contains("/App.tsx"));
        assert!(messages[0].content.contains("export default function App"));
        assert!(messages[1].content.contains("Pages JSON"));
    }
}
