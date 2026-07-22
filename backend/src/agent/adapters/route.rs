use crate::agent::context::AgentContext;
use crate::routes::chat::dto::ChatRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteFlow {
    Traditional,
}

#[derive(Debug, Clone)]
pub struct RouteResult {
    pub flow: RouteFlow,
    pub context: AgentContext,
    pub route_type: RouteType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteType {
    Prompt,
    Fallback,
}

pub trait RouteAdapter: Send + Sync {
    fn name(&self) -> &'static str;

    fn priority(&self) -> i32;

    fn can_handle(&self, request: &ChatRequest) -> bool;

    fn adapt(&self, request: &ChatRequest) -> anyhow::Result<RouteResult>;
}

pub fn has_text_prompt(request: &ChatRequest) -> bool {
    request.messages.iter().any(|message| {
        message.role == "user"
            && message
                .content
                .as_str()
                .map(str::trim)
                .filter(|content| !content.is_empty())
                .is_some()
    })
}
