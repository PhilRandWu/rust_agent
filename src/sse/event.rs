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
}
