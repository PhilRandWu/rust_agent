use crate::ast::parser::ParsedProgram;
use anyhow::Context;
use swc_core::common::GLOBALS;
use swc_core::common::sync::Lrc;
use swc_core::ecma::ast::EsVersion;
use swc_core::ecma::codegen::text_writer::JsWriter;
use swc_core::ecma::codegen::{Config, Emitter};

pub fn emit_program(parsed: &ParsedProgram) -> anyhow::Result<String> {
    GLOBALS.set(&parsed.globals, || -> anyhow::Result<String> {
        let mut buf: Vec<u8> = Vec::new();
        {
            let writer = JsWriter::new(Lrc::clone(&parsed.source_map), "\n", &mut buf, None);
            let cfg = Config::default()
                .with_target(EsVersion::EsNext)
                .with_minify(false)
                .with_ascii_only(false)
                .with_omit_last_semi(false);

            let mut emitter = Emitter {
                cfg,
                comments: None,
                cm: Lrc::clone(&parsed.source_map),
                wr: writer,
            };
            emitter
                .emit_program(&parsed.program)
                .context("swc emit_program failed")?;
        }
        Ok(String::from_utf8(buf).context("codegen produced nan-utf8")?)
    })
}
