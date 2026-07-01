use crate::agent::flows::traditional::ui::UiInput;
use crate::llm::message::LlmMessage;

pub fn ui_messages(input: &UiInput) -> Vec<LlmMessage> {
    let analysis_json = serde_json::to_string(&input.analysis).unwrap_or_else(|_| "{}".to_string());

    let intent_json = serde_json::to_string(&input.intent).unwrap_or_else(|_| "{}".to_string());

    let capability_json =
        serde_json::to_string(&input.capabilities).unwrap_or_else(|_| "{}".to_string());

    vec![
        LlmMessage::system(
            r#"You are a frontend UI planning expert.
Based on the analysis, intent, and capability JSON, describe the UI strategy.
Return only valid JSON with these fields:
layout_pattern: SINGLE_PAGE | DASHBOARD | SPLIT_PANE | MULTI_PAGE
visual_style: MINIMAL | MODERN | PLAYFUL | PROFESSIONAL
primary_sections: string[]
interactions: string[]
responsive_notes: string[]"#,
        ),
        LlmMessage::user(format!(
            "Analysis JSON: {}\nIntent JSON: {}\nCapability JSON: {}",
            analysis_json, intent_json, capability_json
        )),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::analysis::AnalysisOutput;
    use crate::agent::flows::traditional::capability::schema::CapabilityPriority;
    use crate::agent::flows::traditional::capability::{CapabilityItem, CapabilityOutput};
    use crate::agent::flows::traditional::intent::{IntentOutput, IntentType};
    use crate::agent::flows::traditional::ui::UiInput;
    use crate::llm::message::LlmRole;

    #[test]
    fn builds_ui_messages_with_analysis_intent_and_capabilities() {
        let messages = ui_messages(&UiInput::new(
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
        ));

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, LlmRole::System);
        assert_eq!(messages[1].role, LlmRole::User);
        assert!(messages[1].content.contains(r#""app_type":"todo""#));
        assert!(
            messages[1]
                .content
                .contains(r#""goal":"Create a todo app""#)
        );
        assert!(messages[1].content.contains(r#""name":"Todo creation""#));
    }
}
