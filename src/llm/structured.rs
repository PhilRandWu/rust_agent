use crate::llm::client::LlmClient;
use crate::llm::message::LlmMessage;
use serde::de::DeserializeOwned;

pub async fn structured_chat<T>(
    client: &dyn LlmClient,
    messages: &[LlmMessage],
) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let response = client.chat(messages).await?;
    parse_structured_content(&response.content)
}

pub fn parse_structured_content<T>(content: &str) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    Ok(serde_json::from_str(normalize_json_content(content))?)
}

fn normalize_json_content(content: &str) -> &str {
    let content = content.trim();

    if let Some(stripped) = content.strip_prefix("```json") {
        return stripped.trim().trim_end_matches("```").trim();
    }

    if let Some(stripped) = content.strip_prefix("```") {
        return stripped.trim().trim_end_matches("```");
    }

    content
}

#[cfg(test)]
mod tests {
    use crate::llm::message::LlmMessage;
    use crate::llm::mock::MockLlmClient;
    use crate::llm::structured::{parse_structured_content, structured_chat};
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct AnalysisOutput {
        summary: String,
        tags: Vec<String>,
    }

    #[tokio::test]
    async fn parses_json_response_into_type() {
        let client = MockLlmClient::new(r#"{"summary":"Todo app","tags":["todo","app"]}"#);

        let output: AnalysisOutput = structured_chat(&client, &[LlmMessage::user("build app")])
            .await
            .expect("structured chat failed");
        assert_eq!(output.summary, "Todo app");
        assert_eq!(output.tags, vec!["todo", "app"]);
    }

    #[test]
    fn returns_error_for_invalid_json() {
        let error = parse_structured_content::<AnalysisOutput>("error json")
            .expect_err("expected json error");

        assert!(error.to_string().contains("expected"));
    }

    #[test]
    fn parses_json_wrapped_in_markdown_fence() {
        let output: AnalysisOutput = parse_structured_content(
            r#"```json
            {"summary":"Todo app","tags":["todo"]}
            "#,
        )
        .expect("structured chat failed");

        assert_eq!(output.summary, "Todo app");
        assert_eq!(output.tags, vec!["todo"]);
    }
}
