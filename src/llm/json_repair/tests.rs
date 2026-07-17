use super::*;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct Sample {
    name: String,
    tags: Vec<String>,
}

fn parse(input: &str) -> Sample {
    parse_with_repair::<Sample>(input)
        .expect("should parse")
        .value
}

#[test]
fn parses_clean_json_without_any_repair() {
    let result = parse_with_repair::<Sample>(r#"{"name":"a","tags":["x"]}"#).unwrap();
    assert!(result.stages_applied.is_empty());
    assert_eq!(result.value.name, "a");
}

#[test]
fn strips_json_code_fence() {
    let raw = "```json\n{\"name\":\"a\",\"tags\":[\"x\"]}\n```";
    let out = parse(raw);
    assert_eq!(out.name, "a");
}

#[test]
fn strips_bare_code_fence() {
    let raw = "```\n{\"name\":\"a\",\"tags\":[\"x\"]}\n```";
    assert_eq!(parse(raw).name, "a");
}

#[test]
fn strips_ts_language_fence() {
    let raw = "```ts\n{\"name\":\"a\",\"tags\":[]}\n```";
    assert_eq!(parse(raw).tags, Vec::<String>::new());
}

#[test]
fn extracts_json_from_leading_preface() {
    let raw = "Sure, here's the JSON you requested:\n{\"name\":\"a\",\"tags\":[]}";
    assert_eq!(parse(raw).name, "a");
}

#[test]
fn extracts_json_from_trailing_explanation() {
    let raw = "{\"name\":\"a\",\"tags\":[]}\nHope this helps!";
    assert_eq!(parse(raw).name, "a");
}

#[test]
fn extracts_json_from_leading_and_trailing_text() {
    let raw = "Here you go:\n{\"name\":\"a\",\"tags\":[\"x\"]}\nDone.";
    assert_eq!(parse(raw).tags, vec!["x"]);
}

#[test]
fn removes_trailing_comma_in_object() {
    let raw = r#"{"name":"a","tags":["x"],}"#;
    assert_eq!(parse(raw).name, "a");
}

#[test]
fn removes_trailing_comma_in_array() {
    let raw = r#"{"name":"a","tags":["x","y",]}"#;
    assert_eq!(parse(raw).tags, vec!["x", "y"]);
}

#[test]
fn balances_missing_closing_curly() {
    let raw = r#"{"name":"a","tags":["x"]"#;
    assert_eq!(parse(raw).name, "a");
}

#[test]
fn balances_missing_closing_bracket_and_curly() {
    let raw = r#"{"name":"a","tags":["x","y"]"#;
    assert_eq!(parse(raw).tags, vec!["x", "y"]);
}

#[test]
fn does_not_silently_rescue_unterminated_string() {
    let raw = r#"{"name":"a","tags":["x"#;
    assert!(
        parse_with_repair::<Sample>(raw).is_err(),
        "unterminated string should not be silently rescued"
    );
}

#[test]
fn strips_bom_prefix() {
    let raw = "\u{feff}{\"name\":\"a\",\"tags\":[]}";
    assert_eq!(parse(raw).name, "a");
}

#[test]
fn combines_fence_and_trailing_comma() {
    let raw = "```json\n{\"name\":\"a\",\"tags\":[\"x\",]}\n```";
    assert_eq!(parse(raw).tags, vec!["x"]);
}

#[test]
fn does_not_break_on_string_with_braces_inside() {
    let raw = r#"{"name":"has } brace","tags":[]}"#;
    assert_eq!(parse(raw).name, "has } brace");
}

#[test]
fn does_not_break_on_string_with_comma_before_close() {
    let raw = r#"{"name":"end,]","tags":[]}"#;
    assert_eq!(parse(raw).name, "end,]");
}

#[test]
fn returns_error_when_completely_non_json() {
    let err = parse_with_repair::<Sample>("Sorry, I cannot help with that.")
        .expect_err("plain prose should fail");
    let msg = err.to_string();
    assert!(msg.contains("failed to parse"));
}

#[test]
fn records_stages_applied_on_success() {
    let raw = "```json\n{\"name\":\"a\",\"tags\":[\"x\",]}\n```";
    let outcome = parse_with_repair::<Sample>(raw).unwrap();
    assert!(outcome.stages_applied.contains(&RepairStage::FenceStripped));
    assert!(
        outcome
            .stages_applied
            .contains(&RepairStage::TrailingCommaRemoved)
    );
}
