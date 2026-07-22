use crate::agent::flows::traditional::dependency::PackageJson;
use std::collections::BTreeMap;

pub fn react_ts_template_package_json() -> PackageJson {
    PackageJson {
        scripts: scripts(),
        dependencies: dependencies(),
        dev_dependencies: dev_dependencies(),
    }
}

fn scripts() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("dev".to_string(), "vite".to_string()),
        ("build".to_string(), "vite build".to_string()),
        ("preview".to_string(), "vite preview".to_string()),
    ])
}

fn dependencies() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("@vitejs/plugin-react".to_string(), "latest".to_string()),
        ("vite".to_string(), "latest".to_string()),
        ("typescript".to_string(), "latest".to_string()),
        ("react".to_string(), "latest".to_string()),
        ("react-dom".to_string(), "latest".to_string()),
        ("lucide-react".to_string(), "latest".to_string()),
        ("clsx".to_string(), "latest".to_string()),
        ("tailwind-merge".to_string(), "latest".to_string()),
    ])
}

fn dev_dependencies() -> BTreeMap<String, String> {
    BTreeMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_react_ts_template_package_json() {
        let package_json = react_ts_template_package_json();

        assert_eq!(package_json.scripts.get("dev"), Some(&"vite".to_string()));
        assert!(package_json.dependencies.contains_key("react"));
        assert!(package_json.dependencies.contains_key("react-dom"));
        assert!(package_json.dependencies.contains_key("vite"));
    }

    #[test]
    fn includes_sandpack_friendly_utility_dependencies() {
        let package_json = react_ts_template_package_json();

        assert!(package_json.dependencies.contains_key("lucide-react"));
        assert!(package_json.dependencies.contains_key("clsx"));
        assert!(package_json.dependencies.contains_key("tailwind-merge"));
    }
}
