use crate::agent::adapters::prompt::PromptRouteAdapter;
use crate::agent::adapters::route::{RouteAdapter, RouteFlow, RouteResult, RouteType};
use crate::agent::context::AgentContext;
use crate::routes::chat::dto::ChatRequest;

pub struct RouteRegistry {
    adapters: Vec<Box<dyn RouteAdapter>>,
}

impl RouteRegistry {
    pub fn new() -> Self {
        let mut adapters: Vec<Box<dyn RouteAdapter>> = vec![Box::new(PromptRouteAdapter)];

        adapters.sort_by_key(|adapter| std::cmp::Reverse(adapter.priority()));

        Self { adapters }
    }

    pub fn resolve(&self, request: &ChatRequest) -> anyhow::Result<RouteResult> {
        for adapter in &self.adapters {
            if adapter.can_handle(request) {
                return adapter.adapt(request);
            }
        }
        self.fallback(request)
    }

    fn fallback(&self, request: &ChatRequest) -> anyhow::Result<RouteResult> {
        Ok(RouteResult {
            flow: RouteFlow::Traditional,
            context: AgentContext::from_chat_request(request)?,
            route_type: RouteType::Fallback,
        })
    }
}

impl Default for RouteRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::agent::adapters::registry::RouteRegistry;
    use crate::agent::adapters::route::{RouteFlow, RouteType};
    use crate::routes::chat::dto::{ChatMessage, ChatRequest};
    use serde_json::json;

    #[test]
    fn resolves_prompt_route() {
        let registry = RouteRegistry::new();

        let request = ChatRequest {
            messages: vec![ChatMessage {
                id: Some("1".to_string()),
                role: "user".to_string(),
                content: json!("build a dashboard"),
                attachments: None,
            }],
            project_id: None,
            mock_config: None,
        };

        let result = registry.resolve(&request).expect("route should resolve");

        assert_eq!(result.flow, RouteFlow::Traditional);
        assert_eq!(result.route_type, RouteType::Prompt);
        assert_eq!(result.context.user_request, "build a dashboard");
    }

    #[test]
    fn fallback_returns_error_when_context_cannot_build() {
        let registry = RouteRegistry::new();

        let request = ChatRequest {
            messages: vec![],
            project_id: None,
            mock_config: None,
        };

        let error = registry
            .resolve(&request)
            .expect_err("empty request should fail");

        assert!(error.to_string().contains("user message"));
    }
}
