use crate::llm::message::LlmMessage;
use crate::routes::chat::dto::ChatRequest;

pub fn chat_request_to_llm_messages(request: &ChatRequest) -> Vec<LlmMessage> {
    request
        .messages
        .iter()
        .filter_map(|message| {
            let content = message.content.as_str()?.trim();

            if content.is_empty() {
                return None;
            }

            match message.role.as_str() {
                "user" => Some(LlmMessage::user(content)),
                "assistant" => Some(LlmMessage::user(content)),
                _ => None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::routes::chat::converter::chat_request_to_llm_messages;
    use crate::routes::chat::dto::{ChatMessage, ChatRequest};
    use serde_json::json;

    #[test]
    fn converts_user_and_assistant_messages() {
        let request = ChatRequest {
            messages: vec![
                ChatMessage {
                    id: Some(String::from("1")),
                    role: "user".to_string(),
                    content: json!("hello"),
                    attachments: None,
                },
                ChatMessage {
                    id: Some(String::from("2")),
                    role: "assistant".to_string(),
                    content: json!("hi"),
                    attachments: None,
                },
            ],
            project_id: Some(String::from("demo")),
            mock_config: None,
        };

        let message = chat_request_to_llm_messages(&request);
        assert_eq!(message.len(), 2);
    }
}
