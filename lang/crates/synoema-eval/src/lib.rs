//! # synoema-eval
//! Tree-walking interpreter for the Synoema programming language.
//!
//! Implements strict (eager) evaluation following the big-step
//! operational semantics from the Language Reference §5.

pub mod value;
pub mod eval;

pub use value::{Value, Env};
pub use eval::{Evaluator, EvalError, EvalErrorKind};

use synoema_diagnostic::{Diagnostic, codes};
use synoema_types::{TypeError, TypeErrorKind};

// ── Error conversion ────────────────────────────────────

fn parse_err(e: &synoema_parser::ParseError) -> Diagnostic {
    Diagnostic::error(codes::PARSE_UNEXPECTED_TOKEN, e.message.clone())
        .with_span(e.span)
}

fn type_err(e: TypeError) -> Diagnostic {
    let code = match &e.kind {
        TypeErrorKind::Mismatch { .. } => codes::TYPE_MISMATCH,
        TypeErrorKind::InfiniteType { .. } => codes::TYPE_INFINITE,
        TypeErrorKind::Unbound { .. } => codes::TYPE_UNBOUND_VAR,
        TypeErrorKind::UnboundType { .. } => codes::TYPE_UNBOUND_TYPE,
        TypeErrorKind::ArityMismatch { .. } => codes::TYPE_ARITY,
        TypeErrorKind::PatternMismatch { .. } => codes::TYPE_PATTERN,
        TypeErrorKind::LinearDuplicate { .. } => codes::LINEAR_DUPLICATE,
        TypeErrorKind::LinearUnused { .. } => codes::LINEAR_UNUSED,
        TypeErrorKind::Other(_) => codes::TYPE_OTHER,
    };
    let mut diag = Diagnostic::error(code, format!("{}", e));
    // Attach structured notes for type mismatches
    if let TypeErrorKind::Mismatch { expected, found } = &e.kind {
        diag = diag
            .with_note(format!("expected: {}", expected))
            .with_note(format!("found: {}", found));
    }
    diag.maybe_span(e.span)
}

fn eval_err(e: EvalError) -> Diagnostic {
    let code = match e.kind {
        EvalErrorKind::Undefined     => codes::EVAL_UNDEFINED,
        EvalErrorKind::NoMatch       => codes::EVAL_NO_MATCH,
        EvalErrorKind::DivisionByZero => codes::EVAL_DIV_ZERO,
        EvalErrorKind::IoError       => codes::EVAL_IO,
        EvalErrorKind::Type          => codes::EVAL_TYPE,
    };
    Diagnostic::error(code, e.message)
}

/// Convert a `TypeError` from the type-checker into a `Diagnostic`.
///
/// Exposed so that callers (e.g. the REPL's `jit_file`) can use the same
/// precise code dispatch as the interpreter path without duplicating logic.
pub fn type_err_to_diagnostic(e: synoema_types::TypeError) -> Diagnostic {
    type_err(e)
}

// ── Public API ──────────────────────────────────────────

/// Parse, type-check, and evaluate a Synoema program.
/// Returns the final environment.
pub fn run(source: &str) -> Result<Env, Diagnostic> {
    let program = synoema_parser::parse(source)
        .map_err(|e| parse_err(&e))?;
    let program = synoema_types::resolve_modules(program);
    let _types = synoema_types::typecheck(source)
        .map_err(type_err)?;
    let mut evaluator = Evaluator::new();
    evaluator.eval_program(&program)
        .map_err(eval_err)
}

/// Parse, type-check, evaluate, and return a specific function's result
/// when called with no arguments (a constant or nullary function).
///
/// Phase 10.1: runs in a 64 MB stack thread so deeply-recursive programs
/// (like euler1 with 999 recursive calls) don't stack-overflow.
pub fn eval_main(source: &str) -> Result<(Value, Vec<String>), Diagnostic> {
    let source = source.to_string();
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024) // 64 MB — handles ~50 000 levels of recursion
        .spawn(move || eval_main_inner(&source))
        .expect("Failed to spawn eval thread")
        .join()
        .expect("Eval thread panicked")
}

fn eval_main_inner(source: &str) -> Result<(Value, Vec<String>), Diagnostic> {
    let program = synoema_parser::parse(source)
        .map_err(|e| parse_err(&e))?;
    let program = synoema_types::resolve_modules(program);
    let _types = synoema_types::typecheck(source)
        .map_err(type_err)?;
    let mut evaluator = Evaluator::new();
    let env = evaluator.eval_program(&program)
        .map_err(eval_err)?;

    // Look for 'main' or the last defined function
    let main_name = program.decls.iter().rev()
        .find_map(|d| match d {
            synoema_parser::Decl::Func { name, .. } => Some(name.clone()),
            _ => None,
        })
        .ok_or_else(|| Diagnostic::error(codes::EVAL_UNDEFINED, "No function defined"))?;

    let val = env.lookup(&main_name)
        .cloned()
        .ok_or_else(|| Diagnostic::error(codes::EVAL_UNDEFINED, format!("Function '{}' not found", main_name)))?;

    // If it's a zero-arg function (constant), evaluate its body
    let result = match &val {
        Value::Func { equations, .. } if equations.iter().all(|eq| eq.pats.is_empty()) => {
            let eq = &equations[0];
            evaluator.eval(&env, &eq.body)
                .map_err(eval_err)?
        }
        other => other.clone(),
    };

    Ok((result, evaluator.output))
}

/// Quick eval: parse + eval an expression (for REPL), skip typechecking
pub fn eval_expr(source: &str) -> Result<Value, Diagnostic> {
    // Wrap as function definition for the parser
    let wrapped = format!("__expr = {}", source);
    let program = synoema_parser::parse(&wrapped)
        .map_err(|e| parse_err(&e))?;
    let mut evaluator = Evaluator::new();
    let env = evaluator.eval_program(&program)
        .map_err(eval_err)?;

    match env.lookup("__expr") {
        Some(Value::Func { equations, .. }) if !equations.is_empty() => {
            let eq = &equations[0];
            if eq.pats.is_empty() {
                evaluator.eval(&env, &eq.body)
                    .map_err(eval_err)
            } else {
                Ok(env.lookup("__expr").unwrap().clone())
            }
        }
        Some(v) => Ok(v.clone()),
        None => Err(Diagnostic::error(codes::EVAL_UNDEFINED, "Expression not found")),
    }
}

#[cfg(test)]
mod tests;
