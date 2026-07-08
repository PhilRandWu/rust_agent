use crate::agent::event::AgentEvent;
use crate::agent::flows::traditional::analysis::schema::AnalysisOutput;
use crate::agent::flows::traditional::files::schema::FilesOutput;
use crate::agent::flows::traditional::plan::schema::PlanOutput;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

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

    #[serde(rename = "intent")]
    Intent,

    #[serde(rename = "capabilities")]
    Capability,

    #[serde(rename = "ui")]
    Ui,

    #[serde(rename = "components")]
    Component,

    #[serde(rename = "structure")]
    Structure,

    #[serde(rename = "dependency")]
    Dependency,

    #[serde(rename = "types")]
    Type,

    #[serde(rename = "utils")]
    Utils,

    #[serde(rename = "mockData")]
    MockData,

    #[serde(rename = "service")]
    Service,

    #[serde(rename = "hooks")]
    Hooks,

    #[serde(rename = "componentsCode")]
    ComponentGen,

    #[serde(rename = "pagesCode")]
    PageGen,

    #[serde(rename = "layouts")]
    Layout,

    #[serde(rename = "styles")]
    StyleGen,

    #[serde(rename = "app")]
    AppGen,

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

    fn assemble_files_event(files: BTreeMap<String, String>) -> Self {
        Self::data(FrontendEventEnum::Files, json!({ "files": files }))
    }

    fn json_event<T>(event_type: FrontendEventEnum, output: T) -> Self
    where
        T: Serialize,
    {
        Self::data(
            event_type,
            serde_json::to_value(output).unwrap_or_else(|error| {
                json!({
                    "serializationError": error.to_string()
                })
            }),
        )
    }

    pub fn from_agent_event(event: AgentEvent) -> Self {
        match event {
            AgentEvent::Analysis(output) => Self::analysis_output(output),
            AgentEvent::Intent(output) => Self::json_event(FrontendEventEnum::Intent, output),
            AgentEvent::Capability(output) => {
                Self::json_event(FrontendEventEnum::Capability, output)
            }
            AgentEvent::Ui(output) => Self::json_event(FrontendEventEnum::Ui, output),
            AgentEvent::Component(output) => Self::json_event(FrontendEventEnum::Component, output),
            AgentEvent::Structure(output) => Self::json_event(FrontendEventEnum::Structure, output),
            AgentEvent::Dependency(output) => {
                Self::json_event(FrontendEventEnum::Dependency, output)
            }
            AgentEvent::Type(output) => Self::json_event(FrontendEventEnum::Type, output),
            AgentEvent::Utils(output) => Self::json_event(FrontendEventEnum::Utils, output),
            AgentEvent::MockData(output) => Self::json_event(FrontendEventEnum::MockData, output),
            AgentEvent::Service(output) => Self::json_event(FrontendEventEnum::Service, output),
            AgentEvent::Hooks(output) => Self::json_event(FrontendEventEnum::Hooks, output),
            AgentEvent::ComponentGen(output) => {
                Self::json_event(FrontendEventEnum::ComponentGen, output)
            }
            AgentEvent::PageGen(output) => Self::json_event(FrontendEventEnum::PageGen, output),
            AgentEvent::Layout(output) => Self::json_event(FrontendEventEnum::Layout, output),
            AgentEvent::StyleGen(output) => Self::json_event(FrontendEventEnum::StyleGen, output),
            AgentEvent::AppGen(output) => Self::json_event(FrontendEventEnum::AppGen, output),
            AgentEvent::Assemble(output) => Self::assemble_files_event(output.files),
            AgentEvent::PostProcess(output) => Self::assemble_files_event(output.files),
            AgentEvent::Plan(output) => Self::plan_output(output),
            AgentEvent::Files(output) => Self::files_output(output),
            AgentEvent::Error(error) => Self::error(error),
            AgentEvent::Done => Self::done(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::agent::event::AgentEvent;
    use crate::agent::flows::traditional::analysis::schema::AnalysisOutput;
    use crate::agent::flows::traditional::plan::schema::{ComponentPlan, PagePlan, PlanOutput};
    use crate::sse::event::{FrontendEvent, FrontendEventEnum};

    #[test]
    fn converts_agent_done_event_to_frontend_event() {
        let event = FrontendEvent::from_agent_event(AgentEvent::Done);

        let data = event.to_sse_data().unwrap();

        assert_eq!(data, r#"{"type":"done"}"#);
    }

    #[test]
    fn converts_agent_error_event_to_frontend_event() {
        let event = FrontendEvent::from_agent_event(AgentEvent::Error("failed".to_string()));

        let data = event.to_sse_data().unwrap();

        assert_eq!(data, r#"{"type":"error","message":"failed"}"#);
    }

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

    #[test]
    fn serializes_all_traditional_types_matching_frontend_flow() {
        use FrontendEventEnum::*;
        let cases: &[(FrontendEventEnum, &str)] = &[
            (Analysis, "analysis"),
            (Intent, "intent"),
            (Capability, "capabilities"),
            (Ui, "ui"),
            (Component, "components"),
            (Structure, "structure"),
            (Dependency, "dependency"),
            (Type, "types"),
            (Utils, "utils"),
            (MockData, "mockData"),
            (Service, "service"),
            (Hooks, "hooks"),
            (ComponentGen, "componentsCode"),
            (PageGen, "pagesCode"),
            (Layout, "layouts"),
            (StyleGen, "styles"),
            (AppGen, "app"),
            (Plan, "plan"),
            (Files, "files"),
            (Done, "done"),
            (Error, "error"),
        ];
        for (variant, expected) in cases {
            let json_value = serde_json::to_string(variant).unwrap();
            assert_eq!(
                json_value,
                format!("\"{expected}\""),
                "variant {variant:?} should serialize to {expected}"
            );
        }
    }

    #[test]
    fn maps_assemble_agent_event_to_files_frontend_event() {
        use crate::agent::flows::traditional::assemble::{AssembleOutput, AssembleStats};
        use std::collections::BTreeMap;

        let mut files = BTreeMap::new();
        files.insert(
            "App.tsx".to_string(),
            "export default () => null;".to_string(),
        );

        let event = FrontendEvent::from_agent_event(AgentEvent::Assemble(AssembleOutput {
            files: files.clone(),
            stats: AssembleStats {
                total_files: 1,
                categories: BTreeMap::new(),
            },
        }));

        let data = event.to_sse_data().unwrap();
        assert!(data.contains(r#""type":"files""#));
        assert!(data.contains(r#""App.tsx""#));
        assert!(!data.contains(r#""stats""#));
    }

    #[test]
    fn maps_post_process_agent_event_to_files_frontend_event() {
        use crate::agent::flows::traditional::post_process::PostProcessOutput;
        use std::collections::BTreeMap;

        let mut files = BTreeMap::new();
        files.insert("App.tsx".to_string(), "// fixed\n".to_string());

        let event = FrontendEvent::from_agent_event(AgentEvent::PostProcess(PostProcessOutput {
            files: files.clone(),
            total_fixes: 1,
        }));

        let data = event.to_sse_data().unwrap();
        assert!(data.contains(r#""type":"files""#));
        assert!(!data.contains(r#""totalFixes""#));
    }
}
