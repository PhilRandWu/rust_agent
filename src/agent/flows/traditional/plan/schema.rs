use crate::agent::flows::traditional::analysis::schema::AnalysisOutput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanInput {
    pub analysis: AnalysisOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanOutput {
    pub project_name: String,
    pub description: String,
    pub pages: Vec<PagePlan>,
    pub components: Vec<ComponentPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PagePlan {
    pub name: String,
    pub route: String,
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComponentPlan {
    pub name: String,
    pub purpose: String,
}

impl PlanInput {
    pub fn new(analysis: AnalysisOutput) -> Self {
        Self { analysis }
    }
}
