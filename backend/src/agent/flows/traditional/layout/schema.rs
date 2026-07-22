use crate::agent::flows::traditional::page_gen::PageGenOutput;
use crate::agent::flows::traditional::structure::StructureOutput;
use crate::agent::flows::traditional::ui::UiOutput;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutInput {
    pub ui: UiOutput,
    pub structure: StructureOutput,
    pub pages: PageGenOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LayoutOutput {
    pub layouts_code: Vec<LayoutCodeFile>,
    pub route_structure: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutCodeFile {
    pub path: String,
    pub content: String,
    pub description: String,
    pub pages: Vec<String>,
}

impl LayoutInput {
    pub fn new(ui: UiOutput, structure: StructureOutput, pages: PageGenOutput) -> Self {
        Self {
            ui,
            structure,
            pages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_layout_output_from_camel_case_json() {
        let json = r#"
        {
          "layoutsCode": [
            {
              "path": "/layouts/MainLayout.tsx",
              "content": "export default function MainLayout() { return <main />; }",
              "description": "Main layout",
              "pages": ["Home"]
            }
          ],
          "routeStructure": {
            "MainLayout": ["Home"]
          }
        }
        "#;

        let output: LayoutOutput = serde_json::from_str(json).unwrap();

        assert_eq!(output.layouts_code.len(), 1);
        assert_eq!(output.layouts_code[0].path, "/layouts/MainLayout.tsx");
        assert_eq!(
            output.route_structure.get("MainLayout"),
            Some(&vec!["Home".to_string()])
        );
    }
}
