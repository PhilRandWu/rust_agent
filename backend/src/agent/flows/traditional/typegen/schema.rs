use crate::agent::flows::traditional::component::ComponentOutput;
use crate::agent::flows::traditional::structure::StructureOutput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeInput {
    pub components: ComponentOutput,
    pub structure: StructureOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypeOutput {
    pub files: Vec<TypeFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TypeFile {
    pub path: String,
    pub code: String,
    pub model_id: String,
}

impl TypeInput {
    pub fn new(components: ComponentOutput, structure: StructureOutput) -> Self {
        Self {
            components,
            structure,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_type_output_from_camel_case_json() {
        let json = r#"
        {
          "files": [
            {
              "path": "/types/todo.ts",
              "code": "export interface Todo { id: string; title: string; completed: boolean; }",
              "modelId": "Todo"
            }
          ]
        }
        "#;

        let output: TypeOutput = serde_json::from_str(json).unwrap();

        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].model_id, "Todo");
        assert!(output.files[0].code.contains("interface Todo"));
    }
}
