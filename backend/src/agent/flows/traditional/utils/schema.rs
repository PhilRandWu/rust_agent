use crate::agent::flows::traditional::structure::StructureOutput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UtilsInput {
    pub structure: StructureOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UtilsOutput {
    pub files: Vec<UtilsFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UtilsFile {
    pub path: String,
    pub code: String,
    pub description: Option<String>,
}

impl UtilsInput {
    pub fn new(structure: StructureOutput) -> Self {
        Self { structure }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_utils_output_from_json() {
        let json = r#"
        {
          "files": [
            {
              "path": "/lib/utils.ts",
              "code": "export function cn() {}",
              "description": "Shared utilities"
            }
          ]
        }
        "#;

        let output: UtilsOutput = serde_json::from_str(json).unwrap();

        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].path, "/lib/utils.ts");
        assert_eq!(
            output.files[0].description,
            Some("Shared utilities".to_string())
        );
    }
}
