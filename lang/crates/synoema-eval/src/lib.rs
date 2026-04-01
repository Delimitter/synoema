//! # synoema-eval
//! Tree-walking interpreter for the Synoema programming language.
//!
//! Implements strict (eager) evaluation following the big-step
//! operational semantics from the Language Reference §5.

pub mod value;
pub mod eval;

pub use value::{Value, Env};
pub use eval::{Evaluator, EvalError};

/// Parse, type-check, and evaluate an Synoema program.
/// Returns the final environment.
pub fn run(source: &str) -> Result<Env, String> {
    let program = synoema_parser::parse(source)
        .map_err(|e| format!("Parse error: {}", e))?;
    let _types = synoema_types::typecheck(source)
        .map_err(|e| format!("{}", e))?;
    let mut evaluator = Evaluator::new();
    evaluator.eval_program(&program)
        .map_err(|e| format!("{}", e))
}

/// Parse, type-check, evaluate, and return a specific function's result
/// when called with no arguments (a constant or nullary function).
pub fn eval_main(source: &str) -> Result<(Value, Vec<String>), String> {
    let program = synoema_parser::parse(source)
        .map_err(|e| format!("Parse error: {}", e))?;
    let _types = synoema_types::typecheck(source)
        .map_err(|e| format!("{}", e))?;
    let mut evaluator = Evaluator::new();
    let env = evaluator.eval_program(&program)
        .map_err(|e| format!("{}", e))?;

    // Look for 'main' or the last defined function
    let main_name = program.decls.iter().rev()
        .find_map(|d| match d {
            synoema_parser::Decl::Func { name, .. } => Some(name.clone()),
            _ => None,
        })
        .ok_or("No function defined")?;

    let val = env.lookup(&main_name)
        .cloned()
        .ok_or(format!("Function '{}' not found", main_name))?;

    // If it's a zero-arg function (constant), evaluate its body
    let result = match &val {
        Value::Func { equations, .. } if equations.iter().all(|eq| eq.pats.is_empty()) => {
            let eq = &equations[0];
            evaluator.eval(&env, &eq.body)
                .map_err(|e| format!("{}", e))?
        }
        other => other.clone(),
    };

    Ok((result, evaluator.output))
}

/// Quick eval: parse + eval an expression (for REPL), skip typechecking
pub fn eval_expr(source: &str) -> Result<Value, String> {
    // Wrap as function definition for the parser
    let wrapped = format!("__expr = {}", source);
    let program = synoema_parser::parse(&wrapped)
        .map_err(|e| format!("Parse error: {}", e))?;
    let mut evaluator = Evaluator::new();
    let env = evaluator.eval_program(&program)
        .map_err(|e| format!("{}", e))?;

    match env.lookup("__expr") {
        Some(Value::Func { equations, .. }) if !equations.is_empty() => {
            let eq = &equations[0];
            if eq.pats.is_empty() {
                evaluator.eval(&env, &eq.body)
                    .map_err(|e| format!("{}", e))
            } else {
                Ok(env.lookup("__expr").unwrap().clone())
            }
        }
        Some(v) => Ok(v.clone()),
        None => Err("Expression not found".into()),
    }
}

#[cfg(test)]
mod tests;
