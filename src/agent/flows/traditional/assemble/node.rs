use crate::agent::flows::traditional::assemble::package::package_json_content;
use crate::agent::flows::traditional::assemble::{AssembleInput, AssembleOutput, AssembleStats};
use std::collections::BTreeMap;

pub struct AssembleNode;

impl AssembleNode {
    pub fn new() -> Self {
        Self
    }

    #[tracing::instrument(skip_all, name = "node.assemble")]
    pub async fn run(&self, input: AssembleInput) -> anyhow::Result<AssembleOutput> {
        let mut files = BTreeMap::new();
        let mut categories = BTreeMap::new();

        add_file(
            &mut files,
            &mut categories,
            "/package.json",
            package_json_content(&input.dependency)?,
            "config",
        );
        add_file(
            &mut files,
            &mut categories,
            &input.app.path,
            input.app.content,
            "app",
        );
        add_file(
            &mut files,
            &mut categories,
            "/styles.css",
            input.styles.content,
            "styles",
        );

        for file in input.types.files {
            add_file(&mut files, &mut categories, &file.path, file.code, "types");
        }
        for file in input.utils.files {
            add_file(&mut files, &mut categories, &file.path, file.code, "utils");
        }
        for file in input.mock_data.files {
            add_file(
                &mut files,
                &mut categories,
                &file.path,
                file.content,
                "data",
            );
        }
        for file in input.service.files {
            add_file(
                &mut files,
                &mut categories,
                &file.path,
                file.content,
                "services",
            );
        }
        for file in input.hooks.files {
            add_file(
                &mut files,
                &mut categories,
                &file.path,
                file.content,
                "hooks",
            );
        }
        for file in input.component_code.files {
            add_file(
                &mut files,
                &mut categories,
                &file.path,
                file.content,
                "components",
            );
        }
        for file in input.pages.files {
            add_file(
                &mut files,
                &mut categories,
                &file.path,
                file.content,
                "pages",
            );
        }
        for file in input.layouts.layouts_code {
            add_file(
                &mut files,
                &mut categories,
                &file.path,
                file.content,
                "layouts",
            );
        }

        Ok(AssembleOutput {
            stats: AssembleStats {
                total_files: files.len(),
                categories,
            },
            files,
        })
    }
}

impl Default for AssembleNode {
    fn default() -> Self {
        Self::new()
    }
}

fn add_file(
    files: &mut BTreeMap<String, String>,
    categories: &mut BTreeMap<String, usize>,
    path: &str,
    content: String,
    category: &str,
) {
    files.insert(normalize_path(path), content);
    *categories.entry(category.to_string()).or_insert(0) += 1;
}

fn normalize_path(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::app_gen::AppGenOutput;
    use crate::agent::flows::traditional::component_gen::ComponentGenOutput;
    use crate::agent::flows::traditional::dependency::{DependencyOutput, PackageJson};
    use crate::agent::flows::traditional::hooks::HooksOutput;
    use crate::agent::flows::traditional::layout::LayoutOutput;
    use crate::agent::flows::traditional::mock_data::MockDataOutput;
    use crate::agent::flows::traditional::page_gen::PageGenOutput;
    use crate::agent::flows::traditional::service::ServiceOutput;
    use crate::agent::flows::traditional::style_gen::StyleGenOutput;
    use crate::agent::flows::traditional::typegen::TypeOutput;
    use crate::agent::flows::traditional::utils::UtilsOutput;

    #[tokio::test]
    async fn assembles_sandpack_files() {
        let node = AssembleNode::new();

        let output = node
            .run(AssembleInput::new(
                DependencyOutput::new(
                    PackageJson {
                        scripts: BTreeMap::from([("dev".to_string(), "vite".to_string())]),
                        dependencies: BTreeMap::from([("react".to_string(), "latest".to_string())]),
                        dev_dependencies: BTreeMap::new(),
                    },
                    BTreeMap::new(),
                    "base",
                ),
                TypeOutput { files: Vec::new() },
                UtilsOutput { files: Vec::new() },
                MockDataOutput { files: Vec::new() },
                ServiceOutput { files: Vec::new() },
                HooksOutput { files: Vec::new() },
                ComponentGenOutput { files: Vec::new() },
                PageGenOutput { files: Vec::new() },
                LayoutOutput {
                    layouts_code: Vec::new(),
                    route_structure: BTreeMap::new(),
                },
                StyleGenOutput {
                    path: "styles.css".to_string(),
                    content: "body { margin: 0; }".to_string(),
                    description: "styles".to_string(),
                },
                AppGenOutput {
                    path: "/App.tsx".to_string(),
                    content: "export default function App() { return <main />; }".to_string(),
                    description: "app".to_string(),
                },
            ))
            .await
            .expect("assemble node should run");

        assert!(output.files.contains_key("/package.json"));
        assert!(output.files.contains_key("/App.tsx"));
        assert!(output.files.contains_key("/styles.css"));
        assert!(!output.files.contains_key("/src/main.tsx"));
        assert_eq!(output.stats.categories.get("app"), Some(&1));
    }
}
