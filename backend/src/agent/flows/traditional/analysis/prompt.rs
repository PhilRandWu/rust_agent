use crate::agent::flows::traditional::analysis::schema::AnalysisInput;
use crate::llm::message::LlmMessage;

pub fn analysis_message(input: &AnalysisInput) -> Vec<LlmMessage> {
    vec![
        LlmMessage::system(
            r#"You are an app generation requirement analyst.
Return only valid JSON with these fields:
app_type: string
summary: string
features: string[]
pages: string[]
components: string[]"#,
        ),
        LlmMessage::user(format!("Analyze this app request: {}", input.user_request)),
    ]
}
