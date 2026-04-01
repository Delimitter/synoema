//! # synoema-core
//! Core IR and desugaring for the Synoema programming language.
//!
//! Transforms the surface AST into a simplified System F-like
//! intermediate representation suitable for LLVM code generation.

pub mod core_ir;
pub mod desugar;

pub use core_ir::*;
pub use desugar::desugar_program;

/// Parse and desugar Synoema source code into Core IR.
pub fn to_core(source: &str) -> Result<CoreProgram, String> {
    let program = synoema_parser::parse(source)
        .map_err(|e| format!("Parse error: {}", e))?;
    Ok(desugar_program(&program))
}

#[cfg(test)]
mod tests;
