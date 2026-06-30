use crate::routes::chat::dto::ChatRequest;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct AgentContext {
    pub project_id: Option<String>,
    pub mock_config: Option<Value>,
    pub user_request: String,
}

impl AgentContext {
    pub fn from_chat_request(request: &ChatRequest) -> anyhow::Result<Self> {
        let user_request = request
            .messages
            .iter()
            .rev()
            .find(|message| message.role == "user")
            .and_then(|message| message.content.as_str())
            .map(str::trim)
            .filter(|content| !content.is_empty())
            .ok_or_else(|| anyhow::anyhow!("chat request must contain a user message"))?;

        Ok(Self {
            project_id: request.project_id.clone(),
            mock_config: request.mock_config.clone(),
            user_request: user_request.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::agent::context::AgentContext;
    use crate::routes::chat::dto::{ChatMessage, ChatRequest};
    use serde_json::json;

    #[test]
    fn builds_context_from_last_user_message() {
        let request = ChatRequest {
            messages: vec![
                ChatMessage {
                    id: Some("1".to_string()),
                    role: "user".to_string(),
                    content: json!("first request"),
                    attachments: None,
                },
                ChatMessage {
                    id: Some("2".to_string()),
                    role: "assistant".to_string(),
                    content: json!("assistant question"),
                    attachments: None,
                },
                ChatMessage {
                    id: Some("3".to_string()),
                    role: "user".to_string(),
                    content: json!("build a todo app"),
                    attachments: None,
                },
            ],
            project_id: Some("project-1".to_string()),
            mock_config: Some(json!({ "analysisNode": true })),
        };

        let context = AgentContext::from_chat_request(&request).expect("context should work");

        assert_eq!(context.project_id.as_deref(), Some("project-1"));
        assert_eq!(context.user_request, "build a todo app");
        assert_eq!(context.mock_config, Some(json!({ "analysisNode": true })));
    }

    #[test]
    fn rejects_request_without_user_message() {
        let request = ChatRequest {
            messages: vec![ChatMessage {
                id: Some("1".to_string()),
                role: "assistant".to_string(),
                content: json!("hello"),
                attachments: None,
            }],
            project_id: None,
            mock_config: None,
        };

        let error = AgentContext::from_chat_request(&request)
            .expect_err("context should reject missing user message");

        assert!(error.to_string().contains("user message"));
    }
}
