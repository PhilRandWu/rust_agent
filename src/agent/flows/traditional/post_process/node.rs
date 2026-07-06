use crate::agent::flows::traditional::post_process::fixer::fix_files;
use crate::agent::flows::traditional::post_process::{PostProcessInput, PostProcessOutput};

pub struct PostProcessNode;

impl PostProcessNode {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self, input: PostProcessInput) -> anyhow::Result<PostProcessOutput> {
        let (files, total_fixes) = fix_files(input.assembled.files);

        Ok(PostProcessOutput { files, total_fixes })
    }
}

impl Default for PostProcessNode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::assemble::{AssembleOutput, AssembleStats};
    use std::collections::BTreeMap;

    #[tokio::test]
    async fn post_processes_assembled_files() {
        let node = PostProcessNode::new();

        let output = node
            .run(PostProcessInput::new(AssembleOutput {
                files: BTreeMap::from([(
                    "/App.tsx".to_string(),
                    "```tsx\nexport default function App() {}\n```".to_string(),
                )]),
                stats: AssembleStats {
                    total_files: 1,
                    categories: BTreeMap::new(),
                },
            }))
            .await
            .expect("post process should run");

        assert_eq!(
            output.files.get("/App.tsx"),
            Some(&"export default function App() {}".to_string())
        );
        assert!(output.total_fixes >= 1);
    }
}
