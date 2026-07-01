use crate::agent::flows::traditional::plan::schema::PlanInput;
use crate::llm::message::LlmMessage;

pub fn plan_messages(input: &PlanInput) -> Vec<LlmMessage> {
    let analysis_json = serde_json::to_string(&input.analysis).unwrap_or_else(|_| "{}".to_string());

    vec![
        LlmMessage::system(
            r#"You are a frontend application planner.
Based on the analysis JSON, create a concise implementation plan.
Return only valid JSON with these fields:
project_name: string
description: string
pages: [{ name: string, route: string, purpose: string }]
components: [{ name: string, purpose: string }]"#,
        ),
        LlmMessage::user(format!(
            "Create an implementation plan from this analysis JSON: {}",
            analysis_json
        )),
    ]
}
