use crate::agent::flows::traditional::component::ComponentOutput;
use crate::agent::flows::traditional::component_gen::ComponentGenOutput;
use crate::agent::flows::traditional::hooks::HooksOutput;
use crate::agent::flows::traditional::structure::StructureOutput;
use crate::agent::flows::traditional::typegen::TypeOutput;
use crate::agent::flows::traditional::ui::UiOutput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageGenInput {
    pub structure: StructureOutput,
    pub ui: UiOutput,
    pub components: ComponentOutput,
    pub component_code: ComponentGenOutput,
    pub hooks: HooksOutput,
    pub types: TypeOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageGenOutput {
    pub files: Vec<PageCodeFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageCodeFile {
    pub path: String,
    pub content: String,
    pub description: String,
}

impl PageGenInput {
    pub fn new(
        structure: StructureOutput,
        ui: UiOutput,
        components: ComponentOutput,
        component_code: ComponentGenOutput,
        hooks: HooksOutput,
        types: TypeOutput,
    ) -> Self {
        Self {
            structure,
            ui,
            components,
            component_code,
            hooks,
            types,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_page_gen_output_from_json() {
        let json = r#"
        {
          "files": [
            {
              "path": "/pages/Home.tsx",
              "content": "export default function Home() { return <main />; }",
              "description": "Home page"
            }
          ]
        }
        "#;

        let output: PageGenOutput = serde_json::from_str(json).unwrap();

        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].path, "/pages/Home.tsx");
        assert!(output.files[0].content.contains("Home"));
    }
}
