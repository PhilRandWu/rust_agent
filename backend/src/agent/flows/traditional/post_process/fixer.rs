use std::collections::BTreeMap;

pub fn fix_files(files: BTreeMap<String, String>) -> (BTreeMap<String, String>, usize) {
    let mut fixed_files = BTreeMap::new();
    let mut total_fixes = 0;

    for (path, content) in files {
        let (fixed_content, fixes) = fix_content(&content);
        fixed_files.insert(path, fixed_content);
        total_fixes += fixes;
    }

    (fixed_files, total_fixes)
}

pub fn fix_content(content: &str) -> (String, usize) {
    let mut fixes = 0;
    let mut fixed = content.to_string();

    if fixed.contains("```") {
        fixed = fixed.replace("```tsx", "");
        fixed = fixed.replace("```ts", "");
        fixed = fixed.replace("```json", "");
        fixed = fixed.replace("```", "");
        fixes += 1;
    }

    let normalized = fixed.trim().to_string();
    if normalized != fixed {
        fixes += 1;
    }

    (normalized, fixes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_markdown_code_fences() {
        let (content, fixes) = fix_content("```tsx\nexport default function App() {}\n```");

        assert_eq!(content, "export default function App() {}");
        assert!(fixes >= 1);
    }

    #[test]
    fn leaves_normal_content_unchanged() {
        let (content, fixes) = fix_content("export default function App() {}");

        assert_eq!(content, "export default function App() {}");
        assert_eq!(fixes, 0);
    }
}
