//! # synoema-types
//! Hindley-Milner type inference for the Synoema programming language.

pub mod types;
pub mod unify;
pub mod error;
pub mod infer;
pub mod modules;

pub use types::*;
pub use error::TypeError;
pub use infer::Infer;
pub use modules::resolve_modules;

/// Type-check a complete Synoema program.
pub fn typecheck(source: &str) -> Result<TypeEnv, String> {
    let program = synoema_parser::parse(source)
        .map_err(|e| format!("Parse error: {}", e))?;
    let program = resolve_modules(program);
    let mut inf = Infer::new();
    inf.infer_program(&program)
        .map_err(|e| format!("Type error: {}", e))
}

/// Infer the type of a single expression.
pub fn infer_expr_type(source: &str) -> Result<Type, String> {
    let program = synoema_parser::parse(source)
        .map_err(|e| format!("Parse error: {}", e))?;
    let program = resolve_modules(program);
    let mut inf = Infer::new();
    let env = inf.infer_program(&program)
        .map_err(|e| format!("Type error: {}", e))?;

    // Return the type of the last function defined
    if let Some(synoema_parser::Decl::Func { name, .. }) = program.decls.last() {
        if let Some(scheme) = env.lookup(name) {
            return Ok(scheme.ty.clone());
        }
    }
    Err("No function found".into())
}

#[cfg(test)]
mod tests;
