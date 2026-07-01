use crate::agent::flows::traditional::analysis::AnalysisOutput;
use crate::agent::flows::traditional::intent::IntentOutput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityInput {
    pub analysis: AnalysisOutput,
    pub intent: IntentOutput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityOutput {
    pub capabilities: Vec<CapabilityItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityItem {
    pub name: String,
    pub description: String,
    pub priority: CapabilityPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CapabilityPriority {
    MustHave,
    ShouldHave,
    NiceToHave,
}

impl CapabilityInput {
    pub fn new(analysis: AnalysisOutput, intent: IntentOutput) -> Self {
        Self { analysis, intent }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::intent::IntentType;

    #[test]
    fn builds_capability_input() {
        let input = CapabilityInput::new(
            AnalysisOutput {
                app_type: "todo".to_string(),
                summary: "A todo app".to_string(),
                features: vec!["create todos".to_string()],
                pages: vec!["home".to_string()],
                components: vec!["TodoList".to_string()],
            },
            IntentOutput {
                intent_type: IntentType::Create,
                goal: "Create a todo app".to_string(),
                constraints: vec!["Use React".to_string()],
                user_stories: vec!["As a user, I can create todos".to_string()],
            },
        );

        assert_eq!(input.analysis.app_type, "todo");
        assert_eq!(input.intent.goal, "Create a todo app");
    }

    #[test]
    fn parses_capability_priority_from_screaming_snake_case() {
        let priority: CapabilityPriority =
            serde_json::from_str(r#""MUST_HAVE""#).expect("priority should parse");

        assert_eq!(priority, CapabilityPriority::MustHave);
    }
}
