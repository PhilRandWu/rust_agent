use anyhow::Context;
use std::default::Default;
use swc_core::common::sync::Lrc;
use swc_core::common::{FileName, GLOBALS, Globals, SourceMap};
use swc_core::ecma::ast::Program;
use swc_core::ecma::parser::{Parser, StringInput, Syntax, TsSyntax};

pub struct ParsedProgram {
    pub program: Program,
    pub source_map: Lrc<SourceMap>,
    pub globals: Lrc<Globals>,
}

pub fn parse_tsx(source: &str, filename: impl Into<String>) -> anyhow::Result<ParsedProgram> {
    let cm: Lrc<SourceMap> = Lrc::default();
    let fm = cm.new_source_file(
        Lrc::new(FileName::Custom(filename.into())),
        source.to_string(),
    );

    let syntax = Syntax::Typescript(TsSyntax {
        tsx: true,
        decorators: false,
        no_early_errors: true,
        ..Default::default()
    });

    let globals = Lrc::new(Globals::default());
    let program = GLOBALS
        .set(&globals, || -> anyhow::Result<Program> {
            let mut parser = Parser::new(syntax, StringInput::from(&*fm), None);
            let module = parser
                .parse_program()
                .map_err(|e| anyhow::anyhow!("swc parse error: {:?}", e.into_kind()))?;

            let error: Vec<String> = parser
                .take_errors()
                .into_iter()
                .map(|e| format!("{:?}", e.into_kind()))
                .collect();

            if !error.is_empty() {
                tracing::warn!(
                    target: "ast::parser",
                    error= ?error,
                    "swc parser produced non-fatal errors",
                );
            }
            Ok(module)
        })
        .context("parse_tsx failed")?;
    Ok(ParsedProgram {
        program,
        source_map: cm,
        globals,
    })
}

pub fn is_parseable(source: &str) -> bool {
    parse_tsx(source, "<probe>.tsx").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_tsx() {
        assert!(is_parseable(
            "export default function A() { return <div />; "
        ));
    }

    #[test]
    fn parses_tsx_with_imports_and_hooks() {
        let src = r#"
            import { useState } from "react";
            import type { Todo } from "../types/todo";
            export default function TodoList({ todos }: { todos: Todo[]}) {
               const [count, setCount] = useState(0);
               return <ul>{todos.map(t => <li key={t.id}>{t.title}</li>)}</ul>;
            }
        "#;
        assert!(is_parseable(src));
    }

    #[test]
    fn rejects_broken_tsx() {
        assert!(!is_parseable("export default function A( {"));
    }

    #[test]
    fn parses_tsx_with_type_annotations_and_generics() {
        let src = r#"
            export interface Props<T> { items: T[]; onSelect?: (item: T) => void; }
            export function List<T>({ items, onSelect}: Props<T>) {
                return <div>{item.length}</div>;
            }
        "#;
        assert!(is_parseable(src));
    }
}
