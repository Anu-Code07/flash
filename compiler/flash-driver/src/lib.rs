//! Flash compiler driver — orchestrates the full pipeline.

use flash_ir::{format_screen_ir, UiIr};
use flash_parser::parse;
use flash_sema::Compiler;
use flash_span::FileId;

pub struct CompileResult {
    pub ir: UiIr,
    pub ir_text: String,
}

pub fn compile(source: &str) -> Result<CompileResult, CompileError> {
    let file = FileId(0);
    let mut parsed = parse(source, file).map_err(|errors| CompileError::Parse(errors))?;

    let mut compiler = Compiler::new();
    let ir = compiler.compile(&parsed.ast, &mut parsed.interner).ok_or_else(|| {
        CompileError::Semantic(compiler.diagnostics().to_vec())
    })?;

    let interner = &parsed.interner;
    let ir_text = if let Some(screen) = ir.screens.first() {
        format_screen_ir(screen, interner)
    } else {
        String::new()
    };

    Ok(CompileResult { ir, ir_text })
}

#[derive(Debug)]
pub enum CompileError {
    Parse(Vec<flash_parser::ParseError>),
    Semantic(Vec<flash_sema::Diagnostic>),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Parse(errors) => {
                for e in errors {
                    writeln!(f, "parse error: {} at {}:{}", e.message, e.span.lo, e.span.hi)?;
                }
            }
            CompileError::Semantic(diags) => {
                for d in diags {
                    writeln!(f, "error[{}]: {}", d.code, d.message)?;
                }
            }
        }
        Ok(())
    }
}

impl std::error::Error for CompileError {}
