use crate::agent::flows::traditional::analysis::AnalysisOutput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentInput {
    pub analysis: AnalysisOutput,
    pub user_request: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntentOutput {
    pub intent_type: IntentType,
    pub goal: String,
    pub constraints: Vec<String>,
    pub user_stories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntentType {
    Create,
    Modify,
    Question,
    Chat,
}

impl IntentInput {
    pub fn new(analysis: AnalysisOutput, user_request: impl Into<String>) -> Self {
        Self {
            analysis,
            user_request: user_request.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_intent_input() {
        let input = IntentInput::new(
            AnalysisOutput {
                app_type: "todo".to_string(),
                summary: "A todo app".to_string(),
                features: vec!["create todos".to_string()],
                pages: vec!["home".to_string()],
                components: vec!["TodoList".to_string()],
            },
            "build a todo app",
        );

        assert_eq!(input.analysis.app_type, "todo");
        assert_eq!(input.user_request, "build a todo app");
    }

    #[test]
    fn parses_intent_type_from_screaming_snake_case() {
        let intent_type: IntentType =
            serde_json::from_str(r#""CREATE""#).expect("intent type should parse");

        assert_eq!(intent_type, IntentType::Create);
    }
}
