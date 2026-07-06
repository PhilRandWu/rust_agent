use crate::agent::flows::traditional::assemble::AssembleOutput;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostProcessInput {
    pub assembled: AssembleOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PostProcessOutput {
    pub files: BTreeMap<String, String>,
    pub total_fixes: usize,
}

impl PostProcessInput {
    pub fn new(assembled: AssembleOutput) -> Self {
        Self { assembled }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_post_process_output_as_camel_case() {
        let output = PostProcessOutput {
            files: BTreeMap::new(),
            total_fixes: 2,
        };

        let json = serde_json::to_string(&output).unwrap();

        assert!(json.contains("totalFixes"));
    }
}
