use crate::agent::flows::traditional::structure::GeneratedBy;
use crate::agent::flows::traditional::utils::schema::{UtilsFile, UtilsInput, UtilsOutput};
use crate::agent::flows::traditional::utils::templates::class_name_utils_code;

pub struct UtilsNode;

impl UtilsNode {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self, input: UtilsInput) -> anyhow::Result<UtilsOutput> {
        let should_generate =
            input.structure.files.iter().any(|file| {
                file.path == "/lib/utils.ts" || file.generated_by == GeneratedBy::Scaffold
            });

        let files = if should_generate {
            vec![UtilsFile {
                path: "/lib/utils.ts".to_string(),
                code: class_name_utils_code(),
                description: Some("Shared className merge utility".to_string()),
            }]
        } else {
            Vec::new()
        };

        Ok(UtilsOutput { files })
    }
}

impl Default for UtilsNode {
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
    async fn generates_utils_file_when_structure_contains_lib_utils() {
        let node = UtilsNode::new();

        let output = node
            .run(UtilsInput::new(StructureOutput {
                files: vec![FileNode {
                    path: "/lib/utils.ts".to_string(),
                    kind: FileKind::New,
                    description: "Shared utilities".to_string(),
                    source_correlation: None,
                    generated_by: GeneratedBy::Scaffold,
                }],
            }))
            .await
            .expect("utils node should run");

        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].path, "/lib/utils.ts");
        assert!(output.files[0].code.contains("export function cn"));
    }

    #[tokio::test]
    async fn skips_utils_file_when_structure_does_not_need_scaffold() {
        let node = UtilsNode::new();

        let output = node
            .run(UtilsInput::new(StructureOutput {
                files: vec![FileNode {
                    path: "/App.tsx".to_string(),
                    kind: FileKind::Overwrite,
                    description: "Application entry".to_string(),
                    source_correlation: None,
                    generated_by: GeneratedBy::AppEntry,
                }],
            }))
            .await
            .expect("utils node should run");

        assert!(output.files.is_empty());
    }
}
