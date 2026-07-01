use crate::agent::flows::traditional::capability::CapabilityInput;
use crate::llm::message::LlmMessage;

pub fn capability_messages(input: &CapabilityInput) -> Vec<LlmMessage> {
    let analysis_json = serde_json::to_string(&input.analysis).unwrap_or_else(|_| "{}".to_string());

    let intent_json = serde_json::to_string(&input.intent).unwrap_or_else(|_| "{}".to_string());

    vec![
        LlmMessage::system(
            r#"You are an application capability analyst.
Based on the analysis JSON and intent JSON, extract the core product capabilities.
Return only valid JSON with these fields:
capabilities: [
  {
    name: string,
    description: string,
    priority: MUST_HAVE | SHOULD_HAVE | NICE_TO_HAVE
  }
]"#,
        ),
        LlmMessage::user(format!(
            "Analysis JSON: {}\nIntent JSON: {}",
            analysis_json, intent_json
        )),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::analysis::AnalysisOutput;
    use crate::agent::flows::traditional::capability::CapabilityInput;
    use crate::agent::flows::traditional::intent::{IntentOutput, IntentType};
    use crate::llm::message::LlmRole;

    #[test]
    fn builds_capability_messages_with_analysis_and_intent() {
        let messages = capability_messages(&CapabilityInput::new(
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
    }
}
