use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AnalysisInput {
    pub user_request: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnalysisOutput {
    pub app_type: String,
    pub summary: String,
    pub features: Vec<String>,
    pub pages: Vec<String>,
    pub components: Vec<String>,
}

impl AnalysisInput {
    pub fn new(user_request: impl Into<String>) -> Self {
        Self {
            user_request: user_request.into(),
        }
    }
}
