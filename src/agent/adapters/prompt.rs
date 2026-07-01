use crate::agent::adapters::route::{
    RouteAdapter, RouteFlow, RouteResult, RouteType, has_text_prompt,
};
use crate::agent::context::AgentContext;
use crate::routes::chat::dto::ChatRequest;

pub struct PromptRouteAdapter;

impl RouteAdapter for PromptRouteAdapter {
    fn name(&self) -> &'static str {
        "prompt-route"
    }

    fn priority(&self) -> i32 {
        70
    }

    fn can_handle(&self, request: &ChatRequest) -> bool {
        has_text_prompt(request)
    }

    fn adapt(&self, request: &ChatRequest) -> anyhow::Result<RouteResult> {
        Ok(RouteResult {
            flow: RouteFlow::Traditional,
            context: AgentContext::from_chat_request(request)?,
            route_type: RouteType::Prompt,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::agent::adapters::prompt::PromptRouteAdapter;
    use crate::agent::adapters::route::{RouteAdapter, RouteFlow, RouteType};
    use crate::routes::chat::dto::{ChatMessage, ChatRequest};
    use serde_json::json;

    #[test]
    fn handles_text_prompt_request() {
        let request = ChatRequest {
            messages: vec![ChatMessage {
                id: Some("1".to_string()),
                role: "user".to_string(),
                content: json!("build a todo app"),
                attachments: None,
            }],
            project_id: Some("project-1".to_string()),
            mock_config: Some(json!({ "analysisNode": true })),
        };

        let adapter = PromptRouteAdapter;

        assert!(adapter.can_handle(&request));
        let result = adapter.adapt(&request).expect("route should adapt");

        assert_eq!(result.flow, RouteFlow::Traditional);
        assert_eq!(result.route_type, RouteType::Prompt);
        assert_eq!(result.context.user_request, "build a todo app");
        assert_eq!(result.context.project_id.as_deref(), Some("project-1"));
    }

    #[test]
    fn rejects_request_without_text_prompt() {
        let request = ChatRequest {
            messages: vec![ChatMessage {
                id: Some("1".to_string()),
                role: "user".to_string(),
                content: json!("   "),
                attachments: None,
            }],
            project_id: None,
            mock_config: None,
        };

        let adapter = PromptRouteAdapter;

        assert!(!adapter.can_handle(&request));
    }
}
