use crate::agent::analysis::AnalysisOutput;
use crate::agent::event::AgentEvent;
use crate::agent::files::FilesOutput;
use crate::agent::plan::PlanOutput;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Serialize)]
pub struct FrontendEvent {
    #[serde(rename = "type")]
    pub event_type: FrontendEventEnum,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum FrontendEventEnum {
    #[serde(rename = "analysis")]
    Analysis,

    #[serde(rename = "plan")]
    Plan,

    #[serde(rename = "files")]
    Files,

    #[serde(rename = "done")]
    Done,

    #[serde(rename = "error")]
    Error,
}

impl FrontendEvent {
    pub fn data(event_type: FrontendEventEnum, data: Value) -> Self {
        Self {
            event_type,
            data: Some(data),
            message: None,
        }
    }

    pub fn done() -> Self {
        Self {
            event_type: FrontendEventEnum::Done,
            data: None,
            message: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            event_type: FrontendEventEnum::Error,
            data: None,
            message: Some(message.into()),
        }
    }

    pub fn placeholder_analysis() -> Self {
        Self::data(
            FrontendEventEnum::Analysis,
            json!({
                "type": "APP_GENERATION",
                "summary": "Rust backend placeholder analysis",
                "tags": ["rust", "placeholder"]
            }),
        )
    }

    pub fn to_sse_data(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(&self)?)
    }

    pub fn analysis_text(summary: impl Into<String>) -> Self {
        Self::data(
            FrontendEventEnum::Analysis,
            json!({
                "type": "APP_GENERATION",
                "summary": summary.into(),
                "tags": ["rust", "llm"]
            }),
        )
    }

    pub fn analysis_output(output: AnalysisOutput) -> Self {
        Self::data(
            FrontendEventEnum::Analysis,
            json!({
                "type": "APP_GENERATION",
                "appType": output.app_type,
                "summary": output.summary,
                "features": output.features,
                "pages": output.pages,
                "components": output.components
            }),
        )
    }

    pub fn plan_output(output: PlanOutput) -> Self {
        Self::data(
            FrontendEventEnum::Plan,
            json!({
                "projectName": output.project_name,
                "description": output.description,
                "pages": output.pages,
                "components": output.components
            }),
        )
    }

    pub fn files_output(output: FilesOutput) -> Self {
        Self::data(
            FrontendEventEnum::Files,
            json!({
                "files": output.files
            }),
        )
    }

    pub fn from_agent_event(event: AgentEvent) -> Self {
        match event {
            AgentEvent::Analysis(output) => Self::analysis_output(output),
            AgentEvent::Plan(output) => Self::plan_output(output),
            AgentEvent::Files(output) => Self::files_output(output),
            AgentEvent::Error(error) => Self::error(error),
            AgentEvent::Done => Self::done(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::agent::analysis::AnalysisOutput;
    use crate::agent::event::AgentEvent;
    use crate::agent::plan::{ComponentPlan, PagePlan, PlanOutput};
    use crate::sse::event::FrontendEvent;

    #[test]
    fn serializes_analysis_text_event() {
        let event = FrontendEvent::analysis_text("hello");

        let data = event.to_sse_data().unwrap();

        assert!(data.contains(r#""type":"analysis""#));
        assert!(data.contains(r#""summary":"hello""#));
    }

    #[test]
    fn converts_agent_analysis_event_to_frontend_event() {
        let event = FrontendEvent::from_agent_event(AgentEvent::Analysis(AnalysisOutput {
            app_type: "todo".to_string(),
            summary: "A todo app".to_string(),
            features: vec!["create todos".to_string()],
            pages: vec!["home".to_string()],
            components: vec!["TodoList".to_string()],
        }));

        let data = event.to_sse_data().unwrap();
        assert!(data.contains(r#""type":"analysis""#));
        assert!(data.contains(r#""appType":"todo""#));
        assert!(data.contains(r#""components":["TodoList"]"#));
    }

    #[test]
    fn converts_agent_plan_event_to_frontend_event() {
        let event = FrontendEvent::from_agent_event(AgentEvent::Plan(PlanOutput {
            project_name: "Todo App".to_string(),
            description: "A todo management application".to_string(),
            pages: vec![PagePlan {
                name: "Home".to_string(),
                route: "/".to_string(),
                purpose: "Display todos".to_string(),
            }],
            components: vec![ComponentPlan {
                name: "TodoList".to_string(),
                purpose: "Render todos".to_string(),
            }],
        }));

        let data = event.to_sse_data().unwrap();

        assert!(data.contains(r#""type":"plan""#));
        assert!(data.contains(r#""projectName":"Todo App""#));
        assert!(data.contains(r#""route":"/""#));
        assert!(data.contains(r#""name":"TodoList""#));
    }
}
