use crate::agent::flows::traditional::component::ComponentOutput;
use crate::agent::flows::traditional::hooks::HooksOutput;
use crate::agent::flows::traditional::service::ServiceOutput;
use crate::agent::flows::traditional::structure::StructureOutput;
use crate::agent::flows::traditional::typegen::TypeOutput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentGenInput {
    pub structure: StructureOutput,
    pub components: ComponentOutput,
    pub types: TypeOutput,
    pub service: ServiceOutput,
    pub hooks: HooksOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComponentGenOutput {
    pub files: Vec<ComponentCodeFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComponentCodeFile {
    pub path: String,
    pub content: String,
    pub description: Option<String>,
}

impl ComponentGenInput {
    pub fn new(
        structure: StructureOutput,
        components: ComponentOutput,
        types: TypeOutput,
        service: ServiceOutput,
        hooks: HooksOutput,
    ) -> Self {
        Self {
            structure,
            components,
            types,
            service,
            hooks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_component_gen_output_from_json() {
        let json = r#"
        {
          "files": [
            {
              "path": "/components/TodoListPanel.tsx",
              "content": "export default function TodoListPanel() { return <div />; }",
              "description": "Todo list component"
            }
          ]
        }
        "#;

        let output: ComponentGenOutput = serde_json::from_str(json).unwrap();

        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].path, "/components/TodoListPanel.tsx");
        assert!(output.files[0].content.contains("TodoListPanel"));
    }
}
