use crate::agent::flows::traditional::layout::LayoutOutput;
use crate::agent::flows::traditional::page_gen::PageGenOutput;
use crate::agent::flows::traditional::structure::StructureOutput;
use crate::agent::flows::traditional::ui::UiOutput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppGenInput {
    pub ui: UiOutput,
    pub structure: StructureOutput,
    pub pages: PageGenOutput,
    pub layouts: LayoutOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppGenOutput {
    pub path: String,
    pub content: String,
    pub description: String,
}

impl AppGenInput {
    pub fn new(
        ui: UiOutput,
        structure: StructureOutput,
        pages: PageGenOutput,
        layouts: LayoutOutput,
    ) -> Self {
        Self {
            ui,
            structure,
            pages,
            layouts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_app_gen_output_from_json() {
        let json = r#"
        {
          "path": "/App.tsx",
          "content": "export default function App() { return <main />; }",
          "description": "Application root"
        }
        "#;

        let output: AppGenOutput = serde_json::from_str(json).unwrap();

        assert_eq!(output.path, "/App.tsx");
        assert!(output.content.contains("function App"));
        assert_eq!(output.description, "Application root");
    }
}
