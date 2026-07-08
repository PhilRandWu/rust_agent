use crate::routes::chat::dto::ChatRequest;
use serde_json::{Value, json};

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

    pub fn ensure_dev_mock_defaults(&mut self, main_is_mock: bool) {
        if !main_is_mock || self.mock_config.is_some() {
            return;
        }
        self.mock_config = Some(json!({
            "analysisNode": true,
            "intentNode": true,
            "capabilityNode": true,
            "uiNode": true,
            "componentNode": true,
            "structureNode": true,
            "dependencyNode": true,
            "typeNode": true,
            "utilsNode": true,
            "mockDataNode": true,
            "serviceNode": true,
            "hooksNode": true,
            "componentGenNode": true,
            "pageGenNode": true,
            "layoutNode": true,
            "styleGenNode": true,
            "appGenNode": true
        }));
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

    #[test]
    fn dev_mock_defaults_fills_when_mock_provider_and_no_mock_config() {
        let mut ctx = AgentContext {
            project_id: None,
            mock_config: None,
            user_request: "build a todo app".to_string(),
        };

        ctx.ensure_dev_mock_defaults(true);

        let cfg = ctx.mock_config.expect("mock_config should be filled");
        assert_eq!(cfg["analysisNode"], json!(true));
        assert_eq!(cfg["appGenNode"], json!(true));
        assert!(cfg.get("assembleNode").is_none());
        assert!(cfg.get("postProcessNode").is_none());
    }

    #[test]
    fn dev_mock_defaults_respects_explicit_config() {
        let mut ctx = AgentContext {
            project_id: None,
            mock_config: Some(json!({ "analysisNode": false })),
            user_request: "build a todo app".to_string(),
        };

        ctx.ensure_dev_mock_defaults(true);

        assert_eq!(
            ctx.mock_config.as_ref().unwrap()["analysisNode"],
            json!(false),
            "explicit mock_config should not be overwritten"
        );
    }

    #[test]
    fn dev_mock_defaults_noop_when_main_is_real_provider() {
        let mut ctx = AgentContext {
            project_id: None,
            mock_config: None,
            user_request: "build a todo app".to_string(),
        };

        ctx.ensure_dev_mock_defaults(false);

        assert!(
            ctx.mock_config.is_none(),
            "real provider should not auto-enable mock defaults"
        );
    }
}
