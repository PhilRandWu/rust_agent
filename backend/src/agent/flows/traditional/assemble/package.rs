use crate::agent::flows::traditional::dependency::{DependencyOutput, PackageJson};
use std::collections::BTreeMap;

pub fn package_json_content(dependency: &DependencyOutput) -> anyhow::Result<String> {
    let package_json = PackageJson {
        scripts: dependency.package_json.scripts.clone(),
        dependencies: merged_dependencies(dependency),
        dev_dependencies: dependency.package_json.dev_dependencies.clone(),
    };

    Ok(serde_json::to_string_pretty(&package_json)?)
}

fn merged_dependencies(dependency: &DependencyOutput) -> BTreeMap<String, String> {
    let mut dependencies = dependency.package_json.dependencies.clone();

    for (name, version) in &dependency.dependencies {
        dependencies.insert(name.clone(), version.clone());
    }

    dependencies
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::dependency::{DependencyOutput, PackageJson};

    #[test]
    fn merges_extra_dependencies_into_package_json() {
        let output = DependencyOutput::new(
            PackageJson {
                scripts: BTreeMap::from([("dev".to_string(), "vite".to_string())]),
                dependencies: BTreeMap::from([("react".to_string(), "latest".to_string())]),
                dev_dependencies: BTreeMap::new(),
            },
            BTreeMap::from([("lucide-react".to_string(), "latest".to_string())]),
            "merged",
        );

        let content = package_json_content(&output).unwrap();

        assert!(content.contains("\"react\""));
        assert!(content.contains("\"lucide-react\""));
        assert!(content.contains("\"dev\""));
    }
}
