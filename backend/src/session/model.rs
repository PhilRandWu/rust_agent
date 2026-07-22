use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Create,
    Edit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VersionSnapshot {
    pub version: u32,
    pub timestamp_ms: u64,
    pub operation: Operation,
    pub prompt: String,
    pub file_count: usize,
    pub files: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VersionMeta {
    pub version: u32,
    pub timestamp_ms: u64,
    pub operation: Operation,
    pub prompt: String,
    pub file_count: usize,
}

impl From<&VersionSnapshot> for VersionMeta {
    fn from(s: &VersionSnapshot) -> Self {
        Self {
            version: s.version,
            timestamp_ms: s.timestamp_ms,
            operation: s.operation,
            prompt: s.prompt.clone(),
            file_count: s.file_count,
        }
    }
}

pub const PROMPT_MAX_CHARS: usize = 500;

pub fn truncate_prompt(input: &str) -> String {
    if input.chars().count() <= PROMPT_MAX_CHARS {
        return input.to_string();
    }
    let mut out: String = input.chars().take(PROMPT_MAX_CHARS).collect();
    out.push_str("…");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_meta_derives_from_snapshot() {
        let snap = VersionSnapshot {
            version: 3,
            timestamp_ms: 100,
            operation: Operation::Edit,
            prompt: "hi".into(),
            file_count: 2,
            files: [("/a.tsx".to_string(), "x".to_string())].into(),
        };
        let meta: VersionMeta = (&snap).into();
        assert_eq!(meta.version, 3);
        assert_eq!(meta.operation, Operation::Edit);
        assert_eq!(meta.file_count, 2);
        assert!(!serde_json::to_string(&meta).unwrap().contains("files"));
    }

    #[test]
    fn truncate_keeps_short_prompt_and_ellipsizes_long() {
        assert_eq!(truncate_prompt("abc"), "abc");
        let long: String = "x".repeat(PROMPT_MAX_CHARS + 10);
        let out = truncate_prompt(&long);
        assert_eq!(out.chars().count(), PROMPT_MAX_CHARS + 1);
        assert!(out.ends_with("…"));
    }

    #[test]
    fn operation_serializes_lowercase() {
        assert_eq!(
            serde_json::to_string(&Operation::Create).unwrap(),
            "\"create\""
        );
        assert_eq!(serde_json::to_string(&Operation::Edit).unwrap(), "\"edit\"");
    }
}
