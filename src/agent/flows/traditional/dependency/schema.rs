use crate::agent::flows::traditional::structure::StructureOutput;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyInput {
    pub structure: StructureOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependencyOutput {
    pub package_json: PackageJson,
    pub dependencies: BTreeMap<String, String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageJson {
    pub scripts: BTreeMap<String, String>,
    pub dependencies: BTreeMap<String, String>,
    pub dev_dependencies: BTreeMap<String, String>,
}

impl DependencyInput {
    pub fn new(structure: StructureOutput) -> Self {
        Self { structure }
    }
}

impl DependencyOutput {
    pub fn new(
        package_json: PackageJson,
        dependencies: BTreeMap<String, String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            package_json,
            dependencies,
            reason: reason.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::structure::{
        FileKind, FileNode, GeneratedBy, StructureOutput,
    };

    #[test]
    fn builds_dependency_input() {
        let input = DependencyInput::new(StructureOutput {
            files: vec![FileNode {
                path: "/App.tsx".to_string(),
                kind: FileKind::Overwrite,
                description: "Application entry component".to_string(),
                source_correlation: None,
                generated_by: GeneratedBy::AppEntry,
            }],
        });

        assert_eq!(input.structure.files.len(), 1);
        assert_eq!(input.structure.files[0].path, "/App.tsx");
    }

    #[test]
    fn serializes_dependency_output_as_camel_case_package_json() {
        let mut scripts = BTreeMap::new();
        scripts.insert("dev".to_string(), "vite".to_string());

        let mut dependencies = BTreeMap::new();
        dependencies.insert("react".to_string(), "^18.2.0".to_string());

        let package_json = PackageJson {
            scripts,
            dependencies,
            dev_dependencies: BTreeMap::new(),
        };

        let output = DependencyOutput::new(
            package_json,
            BTreeMap::new(),
            "Template base dependencies loaded",
        );

        let json = serde_json::to_string(&output).unwrap();

        assert!(json.contains("packageJson"));
        assert!(json.contains("devDependencies"));
        assert!(json.contains("Template base dependencies loaded"));
    }
}
