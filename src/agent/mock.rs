use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockNode {
    Analysis,
    Intent,
    Capability,
    Ui,
    Component,
    Plan,
    Files,
}

impl MockNode {
    pub fn key(self) -> &'static str {
        match self {
            Self::Analysis => "analysisNode",
            Self::Intent => "intentNode",
            Self::Capability => "capabilityNode",
            Self::Ui => "uiNode",
            Self::Component => "componentNode",
            Self::Plan => "planNode",
            Self::Files => "filesNode",
        }
    }

    pub fn file_name(self) -> &'static str {
        match self {
            Self::Analysis => "analysis_result.json",
            Self::Intent => "intent_result.json",
            Self::Capability => "capability_result.json",
            Self::Ui => "ui_result.json",
            Self::Component => "component_result.json",
            Self::Plan => "plan_result.json",
            Self::Files => "files_result.json",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MockStore {
    root: PathBuf,
}

impl MockStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn should_mock(mock_config: Option<&Value>, node: MockNode) -> bool {
        mock_config
            .and_then(|value| value.get(node.key()))
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    pub async fn load<T>(&self, node: MockNode) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
    {
        let path = self.path_for(node);
        let content = fs::read_to_string(&path)?;
        let value = serde_json::from_str::<T>(&content)?;
        Ok(value)
    }

    pub fn path_for(&self, node: MockNode) -> PathBuf {
        self.root.join(node.file_name())
    }
}

impl Default for MockStore {
    fn default() -> Self {
        Self::new("mock")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::mock::MockStore;
    use serde::Deserialize;
    use serde_json::json;

    #[test]
    fn detects_enabled_node_mock() {
        let config = json!({
           "analysisNode": true,
           "planNode": false
        });

        assert!(MockStore::should_mock(Some(&config), MockNode::Analysis));
        assert!(!MockStore::should_mock(Some(&config), MockNode::Plan));
        assert!(!MockStore::should_mock(Some(&config), MockNode::Files));
    }

    #[test]
    fn does_not_mock_without_config() {
        assert!(!MockStore::should_mock(None, MockNode::Analysis));
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct DemoMock {
        name: String,
    }

    #[tokio::test]
    async fn loads_mock_json_from_file() {
        let dir = tempfile::tempdir().expect("temp dir should create");
        let file_path = dir.path().join("analysis_result.json");

        tokio::fs::write(&file_path, r#"{"name":"demo"}"#)
            .await
            .expect("mock file should write");

        let store = MockStore::new(dir.path());
        let value: DemoMock = store
            .load(MockNode::Analysis)
            .await
            .expect("mock file should load");

        assert_eq!(
            value,
            DemoMock {
                name: "demo".to_string()
            }
        );
    }

    #[test]
    fn detects_enabled_intent_mock() {
        let config = serde_json::json!({
            "intentNode": true
        });

        assert!(MockStore::should_mock(Some(&config), MockNode::Intent));
    }
}
