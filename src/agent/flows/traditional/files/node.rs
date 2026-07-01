use crate::agent::flows::traditional::files::schema::{FilesInput, FilesOutput};
use crate::agent::flows::traditional::files::templates::{
    app_tsx, main_tsx, package_json, styles_css,
};
use crate::agent::flows::traditional::plan::schema::PlanOutput;
use std::collections::BTreeMap;

pub struct FilesNode;

impl FilesNode {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self, input: FilesInput) -> anyhow::Result<FilesOutput> {
        Ok(generate_files(input.plan))
    }
}

fn generate_files(plan: PlanOutput) -> FilesOutput {
    let mut files = BTreeMap::new();

    files.insert("package.json".to_string(), package_json(&plan.project_name));

    files.insert(
        "src/App.tsx".to_string(),
        app_tsx(&plan.project_name, &plan.description),
    );

    files.insert("src/main.tsx".to_string(), main_tsx());

    files.insert("src/styles.css".to_string(), styles_css());

    FilesOutput { files }
}

#[cfg(test)]
mod tests {
    use crate::agent::flows::traditional::files::FilesNode;
    use crate::agent::flows::traditional::files::schema::FilesInput;
    use crate::agent::flows::traditional::files::templates::package_json;
    use crate::agent::flows::traditional::plan::schema::{ComponentPlan, PagePlan, PlanOutput};

    #[tokio::test]
    async fn generates_minimum_sandpack_files() {
        let node = FilesNode::new();

        let output = node
            .run(FilesInput::new(PlanOutput {
                project_name: "Todo App".to_string(),
                description: "A todo management application".to_string(),
                pages: vec![PagePlan {
                    name: "Home".to_string(),
                    route: "/".to_string(),
                    purpose: "Display todos".to_string(),
                }],
                components: vec![ComponentPlan {
                    name: "TodoList".to_string(),
                    purpose: "Render todos".to_string(),
                }],
            }))
            .await
            .expect("files node should generate files");

        assert!(output.files.contains_key("package.json"));
        assert!(output.files.contains_key("src/App.tsx"));
        assert!(output.files.contains_key("src/main.tsx"));
        assert!(output.files.contains_key("src/styles.css"));
        assert!(
            output
                .files
                .get("src/App.tsx")
                .unwrap()
                .contains("Todo App")
        );
    }

    #[test]
    fn generates_valid_package_json() {
        let value: serde_json::Value = serde_json::from_str(&package_json("Todo App"))
            .expect("package.json should be valid JSON");

        assert_eq!(value["name"], "todo-app");
        assert_eq!(value["scripts"]["dev"], "vite");
        assert_eq!(value["dependencies"]["react"], "latest");
    }

    #[test]
    fn generates_pretty_package_json() {
        let package_json = package_json("Todo App");

        assert!(package_json.contains("\n  \"scripts\""));
        assert!(package_json.contains("\n  \"name\": \"todo-app\""));
    }

    #[tokio::test]
    async fn files_node_outputs_valid_package_json() {
        let node = FilesNode::new();

        let output = node
            .run(FilesInput::new(PlanOutput {
                project_name: "Todo App".to_string(),
                description: "A todo management application".to_string(),
                pages: vec![],
                components: vec![],
            }))
            .await
            .expect("files node should generate files");

        let package_json = output
            .files
            .get("package.json")
            .expect("package.json should exist");

        let value: serde_json::Value = serde_json::from_str(package_json)
            .expect("files node package.json should be valid JSON");

        assert_eq!(value["name"], "todo-app");
    }
}
