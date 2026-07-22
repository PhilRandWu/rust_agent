use crate::agent::flows::traditional::structure::StructureOutput;
use crate::agent::flows::traditional::typegen::TypeOutput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockDataInput {
    pub types: TypeOutput,
    pub structure: StructureOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MockDataOutput {
    pub files: Vec<MockDataFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MockDataFile {
    pub path: String,
    pub content: String,
    pub description: String,
}

impl MockDataInput {
    pub fn new(types: TypeOutput, structure: StructureOutput) -> Self {
        Self { types, structure }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mock_data_output_from_json() {
        let json = r#"
        {
          "files": [
            {
              "path": "/data/todos.ts",
              "content": "export const MOCK_TODOS = [];",
              "description": "Todo mock data"
            }
          ]
        }
        "#;

        let output: MockDataOutput = serde_json::from_str(json).unwrap();

        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].path, "/data/todos.ts");
        assert!(output.files[0].content.contains("MOCK_TODOS"));
    }
}
