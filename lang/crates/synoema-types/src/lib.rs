// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

//! # synoema-types
//! Hindley-Milner type inference for the Synoema programming language.

pub mod types;
pub mod unify;
pub mod error;
pub mod infer;
pub mod modules;

pub use types::*;
pub use error::{TypeError, TypeErrorKind};
pub use infer::Infer;
pub use modules::resolve_modules;

/// Type-check a complete Synoema program.
///
/// Returns the typed environment on success, or a structured `TypeError`
/// with an optional source span for LLM-friendly error reporting.
pub fn typecheck(source: &str) -> Result<TypeEnv, TypeError> {
    let program = synoema_parser::parse(source)
        .map_err(|e| TypeError::other(format!("Parse error: {}", e)))?;
    let program = resolve_modules(program);
    let mut inf = Infer::new();
    inf.infer_program(&program)
}

/// Type-check a pre-parsed and import-resolved Program.
pub fn typecheck_program(program: &synoema_parser::Program) -> Result<TypeEnv, TypeError> {
    let resolved = resolve_modules(program.clone());
    let mut inf = Infer::new();
    inf.infer_program(&resolved)
}

/// Type-check with error recovery: returns typed environment + all errors found.
/// Useful for LLM workflows where all errors should be reported in one pass.
pub fn typecheck_recovering(source: &str) -> (Result<TypeEnv, TypeError>, Vec<TypeError>) {
    let program = match synoema_parser::parse(source) {
        Ok(p) => p,
        Err(e) => return (Err(TypeError::other(format!("Parse error: {}", e))), vec![]),
    };
    let program = resolve_modules(program);
    let mut inf = Infer::new();
    inf.infer_program_recovering(&program)
}

/// Infer the type of a single expression.
pub fn infer_expr_type(source: &str) -> Result<Type, TypeError> {
    let program = synoema_parser::parse(source)
        .map_err(|e| TypeError::other(format!("Parse error: {}", e)))?;
    let program = resolve_modules(program);
    let mut inf = Infer::new();
    let env = inf.infer_program(&program)?;

    // Return the type of the last function defined
    if let Some(synoema_parser::Decl::Func { name, .. }) = program.decls.last() {
        if let Some(scheme) = env.lookup(name) {
            return Ok(scheme.ty.clone());
        }
    }
    Err(TypeError::other("No function found"))
}

#[cfg(test)]
mod tests;
