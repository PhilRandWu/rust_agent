use crate::agent::flows::traditional::service::ServiceOutput;
use crate::agent::flows::traditional::structure::StructureOutput;
use crate::agent::flows::traditional::typegen::TypeOutput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HooksInput {
    pub types: TypeOutput,
    pub service: ServiceOutput,
    pub structure: StructureOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HooksOutput {
    pub files: Vec<HooksFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HooksFile {
    pub path: String,
    pub content: String,
    pub description: String,
}

impl HooksInput {
    pub fn new(types: TypeOutput, service: ServiceOutput, structure: StructureOutput) -> Self {
        Self {
            types,
            service,
            structure,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hooks_output_from_json() {
        let json = r#"
        {
          "files": [
            {
              "path": "/hooks/useTodos.ts",
              "content": "export function useTodos() { return { todos: [] }; }",
              "description": "Todo hook"
            }
          ]
        }
        "#;

        let output: HooksOutput = serde_json::from_str(json).unwrap();

        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].path, "/hooks/useTodos.ts");
        assert!(output.files[0].content.contains("useTodos"));
    }
}
