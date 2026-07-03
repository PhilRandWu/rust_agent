use crate::agent::flows::traditional::component::ComponentOutput;
use crate::agent::flows::traditional::dependency::DependencyOutput;
use crate::agent::flows::traditional::page_gen::PageGenOutput;
use crate::agent::flows::traditional::ui::UiOutput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleGenInput {
    pub ui: UiOutput,
    pub components: ComponentOutput,
    pub pages: PageGenOutput,
    pub dependency: DependencyOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StyleGenOutput {
    pub path: String,
    pub content: String,
    pub description: String,
}

impl StyleGenInput {
    pub fn new(
        ui: UiOutput,
        components: ComponentOutput,
        pages: PageGenOutput,
        dependency: DependencyOutput,
    ) -> Self {
        Self {
            ui,
            components,
            pages,
            dependency,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_style_gen_output_from_json() {
        let json = r#"
        {
          "path": "styles.css",
          "content": ":root { --color-primary: #2563eb; }",
          "description": "Global styles"
        }
        "#;

        let output: StyleGenOutput = serde_json::from_str(json).unwrap();

        assert_eq!(output.path, "styles.css");
        assert!(output.content.contains(":root"));
        assert_eq!(output.description, "Global styles");
    }
}
