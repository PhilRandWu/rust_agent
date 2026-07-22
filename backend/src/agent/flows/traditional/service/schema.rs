use crate::agent::flows::traditional::mock_data::MockDataOutput;
use crate::agent::flows::traditional::structure::StructureOutput;
use crate::agent::flows::traditional::typegen::TypeOutput;
use crate::agent::flows::traditional::utils::UtilsOutput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceInput {
    pub types: TypeOutput,
    pub mock_data: MockDataOutput,
    pub utils: UtilsOutput,
    pub structure: StructureOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceOutput {
    pub files: Vec<ServiceFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceFile {
    pub path: String,
    pub content: String,
    pub description: String,
}

impl ServiceInput {
    pub fn new(
        types: TypeOutput,
        mock_data: MockDataOutput,
        utils: UtilsOutput,
        structure: StructureOutput,
    ) -> Self {
        Self {
            types,
            mock_data,
            utils,
            structure,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_service_output_from_json() {
        let json = r#"
        {
          "files": [
            {
              "path": "/services/todoService.ts",
              "content": "export function getAllTodos() { return []; }",
              "description": "Todo data access service"
            }
          ]
        }
        "#;

        let output: ServiceOutput = serde_json::from_str(json).unwrap();

        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].path, "/services/todoService.ts");
        assert!(output.files[0].content.contains("getAllTodos"));
    }
}
