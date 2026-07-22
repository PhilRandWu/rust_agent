use crate::agent::flows::traditional::plan::schema::PlanOutput;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesInput {
    pub plan: PlanOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FilesOutput {
    pub files: BTreeMap<String, String>,
}

impl FilesInput {
    pub fn new(plan: PlanOutput) -> FilesInput {
        FilesInput { plan }
    }
}
