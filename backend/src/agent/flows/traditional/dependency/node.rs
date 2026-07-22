use crate::agent::flows::traditional::dependency::schema::{DependencyInput, DependencyOutput};
use crate::agent::flows::traditional::dependency::template::react_ts_template_package_json;
use std::collections::BTreeMap;

pub struct DependencyNode;

impl DependencyNode {
    pub fn new() -> Self {
        Self
    }

    #[tracing::instrument(skip_all, name = "node.dependency")]
    pub async fn run(&self, _input: DependencyInput) -> anyhow::Result<DependencyOutput> {
        Ok(DependencyOutput::new(
            react_ts_template_package_json(),
            BTreeMap::new(),
            "Template base dependencies loaded; import scanning will be handled during assembly.",
        ))
    }
}

impl Default for DependencyNode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::structure::{
        FileKind, FileNode, GeneratedBy, StructureOutput,
    };

    #[tokio::test]
    async fn returns_template_package_json_and_empty_extra_dependencies() {
        let node = DependencyNode::new();

        let output = node
            .run(DependencyInput::new(StructureOutput {
                files: vec![
                    FileNode {
                        path: "/App.tsx".to_string(),
                        kind: FileKind::Overwrite,
                        description: "Application entry".to_string(),
                        source_correlation: None,
                        generated_by: GeneratedBy::AppEntry,
                    },
                    FileNode {
                        path: "/components/TodoInputForm.tsx".to_string(),
                        kind: FileKind::New,
                        description: "Todo input component".to_string(),
                        source_correlation: Some("TodoInputForm".to_string()),
                        generated_by: GeneratedBy::Component,
                    },
                ],
            }))
            .await
            .expect("dependency node should return output");

        assert!(output.package_json.dependencies.contains_key("react"));
        assert!(output.package_json.dependencies.contains_key("react-dom"));
        assert!(output.package_json.dependencies.contains_key("vite"));
        assert!(output.dependencies.is_empty());
        assert!(output.reason.contains("Template base dependencies loaded"));
    }
}
