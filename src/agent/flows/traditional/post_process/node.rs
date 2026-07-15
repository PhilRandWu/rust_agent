use crate::agent::flows::traditional::post_process::ast_fixer::ast_fix_file;
use crate::agent::flows::traditional::post_process::fixer::fix_content;
use crate::agent::flows::traditional::post_process::{PostProcessInput, PostProcessOutput};
use std::collections::BTreeMap;
use tokio::task::JoinSet;

pub struct PostProcessNode;

impl PostProcessNode {
    pub fn new() -> Self {
        Self
    }

    #[tracing::instrument(skip_all, name = "node.post_process")]
    pub async fn run(&self, input: PostProcessInput) -> anyhow::Result<PostProcessOutput> {
        let files = input.assembled.files;
        let mut set: JoinSet<(String, String, usize)> = JoinSet::new();

        for (path, content) in files {
            set.spawn_blocking(move || {
                let (s, sf) = fix_content(&content);
                let (a, af) = ast_fix_file(&path, &s);
                (path, a, sf + af)
            });
        }

        let mut fixed = BTreeMap::new();
        let mut total_fixes = 0;
        while let Some(r) = set.join_next().await {
            let (path, content, fixes) = r?;
            fixed.insert(path, content);
            total_fixes += fixes;
        }

        Ok(PostProcessOutput {
            files: fixed,
            total_fixes,
        })
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

        let content = output.files.get("/App.tsx").expect("App.tsx present");
        assert!(!content.contains("```"), "fences should be gone: {content}");
        assert!(
            content.contains("export default function App"),
            "content preserved: {content}"
        );
        assert!(output.total_fixes >= 1);
    }
}
