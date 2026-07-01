pub fn sanitize_package_name(project_name: &str) -> String {
    let sanitized = project_name
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' {
                ch
            } else if ch.is_whitespace() || ch == '_' {
                '-'
            } else {
                '-'
            }
        })
        .collect::<String>();

    let sanitized = sanitized.trim_matches('-').to_string();

    if sanitized.is_empty() {
        "rust-agent-app".to_string()
    } else {
        sanitized
    }
}

pub fn escape_jsx_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use crate::agent::flows::traditional::files::sanitize::{
        escape_jsx_text, sanitize_package_name,
    };

    #[test]
    fn sanitizes_package_name() {
        assert_eq!(sanitize_package_name("Todo App"), "todo-app");
        assert_eq!(sanitize_package_name("  My_App  "), "my-app");
        assert_eq!(sanitize_package_name("!!!"), "rust-agent-app");
    }

    #[test]
    fn escapes_jsx_text() {
        assert_eq!(
            escape_jsx_text(r#"<script>"x" & 'y'</script>"#),
            "&lt;script&gt;&quot;x&quot; &amp; &#39;y&#39;&lt;/script&gt;"
        );
    }
}
