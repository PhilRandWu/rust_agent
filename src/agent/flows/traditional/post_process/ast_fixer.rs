use crate::ast::codegen::emit_program;
use crate::ast::parser::parse_tsx;
use crate::ast::visitors::imports::remove_unused_imports;

pub fn is_ast_target(path: &str) -> bool {
    path.ends_with(".ts") || path.ends_with(".tsx")
}

pub fn ast_fix_file(path: &str, content: &str) -> (String, usize) {
    if !is_ast_target(path) {
        return (content.to_string(), 0);
    }
    let mut parsed = match parse_tsx(content, path) {
        Ok(p) => p,
        Err(err) => {
            tracing::warn!(
                target: "post_process::ast_fixer",
                path, error = %err,
                "parse failed, falling back to string content",
            );
            return (content.to_string(), 0);
        }
    };
    let removed = remove_unused_imports(&mut parsed.program);
    match emit_program(&parsed) {
        Ok(new_content) => (new_content, removed),
        Err(err) => {
            tracing::warn!(
                target: "post_process::ast_fixer",
                path, error = %err,
                "codegen failed, falling back to string content",
            );
            (content.to_string(), 0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_non_typescript_files() {
        let (out, fixes) = ast_fix_file("/styles.css", "body { margin: 0; }");
        assert_eq!(out, "body { margin: 0; }");
        assert_eq!(fixes, 0);
    }

    #[test]
    fn removes_unused_import_from_tsx() {
        let src = "import { useState, useMemo } from \"react\";\nexport default function A() { return useState(0); }\n";
        let (out, fixes) = ast_fix_file("/components/A.tsx", src);
        assert!(!out.contains("useMemo"), "got: {out}");
        assert!(out.contains("useState"), "got: {out}");
        assert_eq!(fixes, 1);
    }

    #[test]
    fn falls_back_on_parse_error() {
        let broken = "export default function A( {";
        let (out, fixes) = ast_fix_file("/components/A.tsx", broken);
        assert_eq!(out, broken, "content should be untouched on parse error");
        assert_eq!(fixes, 0);
    }

    #[test]
    fn preserves_side_effect_import() {
        let src = "import \"./styles.css\";\nexport const x = 1;\n";
        let (out, _) = ast_fix_file("/x.tsx", src);
        assert!(out.contains("./styles.css"));
    }
}
