use crate::agent::flows::traditional::style_gen::StyleGenInput;
use crate::llm::message::LlmMessage;

pub fn style_gen_messages(input: &StyleGenInput) -> Vec<LlmMessage> {
    let ui_json = serde_json::to_string(&input.ui).unwrap_or_else(|_| "{}".to_string());
    let components_json =
        serde_json::to_string(&input.components).unwrap_or_else(|_| "{}".to_string());
    let pages_json = serde_json::to_string(&input.pages).unwrap_or_else(|_| "{}".to_string());
    let dependency_json =
        serde_json::to_string(&input.dependency).unwrap_or_else(|_| "{}".to_string());

    vec![
        LlmMessage::system(
            r#"You are a senior CSS and design system engineer.

Your task is to generate a global styles.css file for a React + TypeScript Sandpack project.

Return only valid JSON. Do not include markdown.

Output JSON shape:
{
  "path": "styles.css",
  "content": ":root {\n  --color-primary: #2563eb;\n}\n\nbody {\n  margin: 0;\n}\n",
  "description": "Global styles"
}

Rules:
1. path must be styles.css.
2. Generate CSS only.
3. Do not include markdown.
4. Do not include comments.
5. Prefer CSS variables in :root.
6. Include body base styles.
7. Include at least one page container class.
8. Include at least one animation keyframes.
9. Complement Tailwind CSS; do not redefine Tailwind utility classes.
10. Keep CSS concise, around 80 to 150 lines.
11. Do not use @import.
12. Do not include component-specific CSS."#,
        ),
        LlmMessage::user(format!(
            "UI JSON: {}\nComponent JSON: {}\nPage JSON: {}\nDependency JSON: {}",
            ui_json, components_json, pages_json, dependency_json
        )),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::component::ComponentOutput;
    use crate::agent::flows::traditional::dependency::{DependencyOutput, PackageJson};
    use crate::agent::flows::traditional::page_gen::PageGenOutput;
    use crate::agent::flows::traditional::style_gen::StyleGenInput;
    use crate::agent::flows::traditional::ui::{LayoutPattern, UiOutput, VisualStyle};
    use crate::llm::message::LlmRole;
    use std::collections::BTreeMap;

    #[test]
    fn builds_style_gen_messages() {
        let messages = style_gen_messages(&StyleGenInput::new(
            UiOutput {
                layout_pattern: LayoutPattern::SinglePage,
                visual_style: VisualStyle::Modern,
                primary_sections: vec!["Hero".to_string()],
                interactions: Vec::new(),
                responsive_notes: Vec::new(),
            },
            ComponentOutput {
                components: Vec::new(),
            },
            PageGenOutput { files: Vec::new() },
            DependencyOutput::new(
                PackageJson {
                    scripts: BTreeMap::new(),
                    dependencies: BTreeMap::new(),
                    dev_dependencies: BTreeMap::new(),
                },
                BTreeMap::new(),
                "Template dependencies loaded",
            ),
        ));

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, LlmRole::System);
        assert!(messages[0].content.contains("styles.css"));
        assert!(messages[0].content.contains(":root"));
        assert!(messages[1].content.contains("UI JSON"));
    }
}
