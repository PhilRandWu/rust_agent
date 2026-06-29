use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    #[serde(default)]
    pub messages: Vec<ChatMessage>,

    #[serde(rename = "projectId")]
    pub project_id: Option<String>,

    #[serde(rename = "mockConfig")]
    pub mock_config: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct ChatMessage {
    pub id: Option<String>,
    pub role: String,
    pub content: Value,

    #[serde(default)]
    pub attachments: Option<Vec<Value>>,
}
