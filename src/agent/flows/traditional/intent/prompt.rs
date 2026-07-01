use crate::agent::flows::traditional::intent::IntentInput;
use crate::llm::message::LlmMessage;

pub fn intent_messages(input: &IntentInput) -> Vec<LlmMessage> {
    let analysis_json = serde_json::to_string(&input.analysis).unwrap_or_else(|_| "{}".to_string());

    vec![
        LlmMessage::system(
            r#"You are an application intent analyst.
Based on the user request and analysis JSON, extract the user's implementation intent.
Return only valid JSON with these fields:
intent_type: CREATE | MODIFY | QUESTION | CHAT
goal: string
constraints: string[]
user_stories: string[]"#,
        ),
        LlmMessage::user(format!(
            "User request: {}\nAnalysis JSON: {}",
            input.user_request, analysis_json
        )),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::analysis::AnalysisOutput;
    use crate::agent::flows::traditional::intent::IntentInput;
    use crate::llm::message::LlmRole;

    #[test]
    fn builds_intent_messages_with_request_and_analysis() {
        let messages = intent_messages(&IntentInput::new(
            AnalysisOutput {
                app_type: "todo".to_string(),
                summary: "A todo app".to_string(),
                features: vec!["create todos".to_string()],
                pages: vec!["home".to_string()],
                components: vec!["TodoList".to_string()],
            },
            "build a todo app",
        ));

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, LlmRole::System);
        assert_eq!(messages[1].role, LlmRole::User);
        assert!(messages[1].content.contains("build a todo app"));
        assert!(messages[1].content.contains(r#""app_type":"todo""#));
    }
}
