use crate::agent::flows::traditional::analysis::AnalysisOutput;
use crate::agent::flows::traditional::capability::CapabilityOutput;
use crate::agent::flows::traditional::intent::IntentOutput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiInput {
    pub analysis: AnalysisOutput,
    pub intent: IntentOutput,
    pub capabilities: CapabilityOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiOutput {
    pub layout_pattern: LayoutPattern,
    pub visual_style: VisualStyle,
    pub primary_sections: Vec<String>,
    pub interactions: Vec<String>,
    pub responsive_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LayoutPattern {
    SinglePage,
    Dashboard,
    SplitPane,
    MultiPage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VisualStyle {
    Minimal,
    Modern,
    Playful,
    Professional,
}

impl UiInput {
    pub fn new(
        analysis: AnalysisOutput,
        intent: IntentOutput,
        capabilities: CapabilityOutput,
    ) -> Self {
        Self {
            analysis,
            intent,
            capabilities,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::capability::CapabilityItem;
    use crate::agent::flows::traditional::capability::schema::CapabilityPriority;
    use crate::agent::flows::traditional::intent::IntentType;

    #[test]
    fn builds_ui_input() {
        let input = UiInput::new(
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
                constraints: vec!["Keep UI simple".to_string()],
                user_stories: vec!["As a user, I can create todos".to_string()],
            },
            CapabilityOutput {
                capabilities: vec![CapabilityItem {
                    name: "Todo creation".to_string(),
                    description: "Users can create todo items".to_string(),
                    priority: CapabilityPriority::MustHave,
                }],
            },
        );

        assert_eq!(input.analysis.app_type, "todo");
        assert_eq!(input.intent.goal, "Create a todo app");
        assert_eq!(input.capabilities.capabilities.len(), 1);
    }

    #[test]
    fn parses_layout_pattern_from_screaming_snake_case() {
        let layout: LayoutPattern =
            serde_json::from_str(r#""SINGLE_PAGE""#).expect("layout should parse");

        assert_eq!(layout, LayoutPattern::SinglePage);
    }

    #[test]
    fn parses_visual_style_from_screaming_snake_case() {
        let style: VisualStyle = serde_json::from_str(r#""MODERN""#).expect("style should parse");

        assert_eq!(style, VisualStyle::Modern);
    }
}
