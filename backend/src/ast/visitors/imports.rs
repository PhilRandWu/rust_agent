use swc_core::ecma::ast::{
    Id, Ident, ImportDecl, ImportSpecifier, Module, ModuleDecl, ModuleItem, Program, Script,
};
use swc_core::ecma::visit::{Visit, VisitMut, VisitMutWith, VisitWith};

pub fn remove_unused_imports(program: &mut Program) -> usize {
    let mut collector = UsageCollector::default();
    program.visit_with(&mut collector);

    let mut remover = Remover {
        used: collector.used,
        removed: 0,
    };
    program.visit_mut_with(&mut remover);
    remover.removed
}

#[derive(Default)]
struct UsageCollector {
    used: std::collections::HashSet<Id>,
}

impl Visit for UsageCollector {
    fn visit_import_decl(&mut self, _: &ImportDecl) {}

    fn visit_ident(&mut self, ident: &Ident) {
        self.used.insert(ident.to_id());
    }
}

struct Remover {
    used: std::collections::HashSet<Id>,
    removed: usize,
}

impl VisitMut for Remover {
    fn visit_mut_module(&mut self, module: &mut Module) {
        module.body.retain_mut(|item| self.keep_module_item(item));
    }

    fn visit_mut_script(&mut self, _: &mut Script) {}
}

impl Remover {
    fn keep_module_item(&mut self, item: &mut ModuleItem) -> bool {
        let ModuleItem::ModuleDecl(ModuleDecl::Import(decl)) = item else {
            return true;
        };
        self.trim_import_decl(decl)
    }

    fn trim_import_decl(&mut self, decl: &mut ImportDecl) -> bool {
        let before = decl.specifiers.len();

        if before == 0 {
            return true;
        }

        decl.specifiers.retain(|spec| {
            let local = spec_local(spec);
            if local.sym.as_ref() == "React" {
                return true;
            }
            self.used.contains(&local.to_id())
        });
        self.removed += before - decl.specifiers.len();

        !decl.specifiers.is_empty()
    }
}

fn spec_local(spec: &ImportSpecifier) -> &Ident {
    match spec {
        ImportSpecifier::Named(s) => &s.local,
        ImportSpecifier::Default(s) => &s.local,
        ImportSpecifier::Namespace(s) => &s.local,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::codegen::emit_program;
    use crate::ast::parser::parse_tsx;

    fn round_trip(source: &str) -> (String, usize) {
        let mut parsed = parse_tsx(source, "test.tsx").expect("parse");
        let removed = remove_unused_imports(&mut parsed.program);
        let out = emit_program(&parsed).expect("emit");
        (out, removed)
    }

    #[test]
    fn removes_unused_named_import() {
        let src = r#"import { useState, useMemo } from "react";
export default function A() { return useState(0); }
"#;
        let (out, removed) = round_trip(src);
        assert!(
            !out.contains("useMemo"),
            "unused import should be gone: {out}"
        );
        assert!(out.contains("useState"), "used import must remain: {out}");
        assert_eq!(removed, 1);
    }

    #[test]
    fn keeps_react_default_import() {
        let src = r#"import React from "react";
export default function A() { return <div />; }
"#;
        let (out, _) = round_trip(src);
        assert!(out.contains("React"), "React should be kept: {out}");
    }

    #[test]
    fn removes_entire_import_when_all_specifiers_unused() {
        let src = r#"import { unused } from "some-lib";
export const x = 1;
"#;
        let (out, removed) = round_trip(src);
        assert!(!out.contains("some-lib"), "whole import gone: {out}");
        assert_eq!(removed, 1);
    }

    #[test]
    fn keeps_side_effect_import() {
        let src = r#"import "./styles.css";
export const x = 1;
"#;
        let (out, removed) = round_trip(src);
        assert!(
            out.contains("./styles.css"),
            "side-effect import kept: {out}"
        );
        assert_eq!(removed, 0);
    }

    #[test]
    fn keeps_type_only_import_when_used_as_type() {
        let src = r#"import type { Todo } from "../types/todo";
export function f(x: Todo): Todo { return x; }
"#;
        let (out, _) = round_trip(src);
        assert!(out.contains("Todo"), "type-only used import kept: {out}");
    }
}
