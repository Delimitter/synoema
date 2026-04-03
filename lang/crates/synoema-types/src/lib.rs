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
