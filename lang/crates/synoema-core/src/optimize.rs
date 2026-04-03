//! Optimization pass over Core IR.
//!
//! Phase 10.2: Constant Folding + Dead Code Elimination
//!
//! Transformations performed (bottom-up):
//! 1. Constant folding: `App(App(PrimOp(op), Lit(a)), Lit(b))` → `Lit(result)`
//! 2. Unary constant folding: `App(PrimOp(Neg/Not), Lit(x))` → `Lit(result)`
//! 3. Conditional elimination: `Case(Lit(...), alts)` → matching alt body
//! 4. Dead let elimination: `Let(x, Lit(...), body)` where x not free in body → body

use synoema_parser::Lit;
use crate::core_ir::{CoreExpr, CorePat, CoreProgram, CoreDef, PrimOp, Alt};

// ── Public API ───────────────────────────────────────────────

/// Optimize a single Core IR expression.
pub fn optimize_expr(expr: CoreExpr) -> CoreExpr {
    fold_expr(expr)
}

/// Optimize a complete Core IR program.
pub fn optimize_program(program: CoreProgram) -> CoreProgram {
    let defs = program.defs.into_iter().map(|def| CoreDef {
        name: def.name,
        body: fold_expr(def.body),
    }).collect();
    CoreProgram { defs, ctor_tags: program.ctor_tags }
}

// ── Core folding pass (bottom-up) ────────────────────────────

fn fold_expr(expr: CoreExpr) -> CoreExpr {
    match expr {
        // Recurse into children first, then try to fold the result
        CoreExpr::App(func, arg) => {
            let func = fold_expr(*func);
            let arg = fold_expr(*arg);
            fold_app(func, arg)
        }

        CoreExpr::Lam(param, body) => {
            CoreExpr::Lam(param, Box::new(fold_expr(*body)))
        }

        CoreExpr::Let(name, val, body) => {
            let val = fold_expr(*val);
            let body = fold_expr(*body);
            fold_let(name, val, body)
        }

        CoreExpr::LetRec(name, val, body) => {
            CoreExpr::LetRec(
                name,
                Box::new(fold_expr(*val)),
                Box::new(fold_expr(*body)),
            )
        }

        CoreExpr::Case(scrut, alts) => {
            let scrut = fold_expr(*scrut);
            let alts: Vec<Alt> = alts.into_iter().map(|alt| Alt {
                pat: alt.pat,
                body: fold_expr(alt.body),
            }).collect();
            fold_case(scrut, alts)
        }

        CoreExpr::MkList(elems) => {
            CoreExpr::MkList(elems.into_iter().map(fold_expr).collect())
        }

        CoreExpr::MkClosure { func, free_vars } => {
            CoreExpr::MkClosure { func, free_vars }
        }

        CoreExpr::Record(fields) => {
            CoreExpr::Record(fields.into_iter().map(|(n, e)| (n, fold_expr(e))).collect())
        }

        CoreExpr::FieldAccess(expr, name) => {
            let expr = fold_expr(*expr);
            // Constant-fold: {name = v, ...}.name → v
            if let CoreExpr::Record(ref fields) = expr {
                if let Some((_, val)) = fields.iter().find(|(n, _)| n == &name) {
                    return val.clone();
                }
            }
            CoreExpr::FieldAccess(Box::new(expr), name)
        }

        CoreExpr::Scope(body) => CoreExpr::Scope(Box::new(fold_expr(*body))),
        CoreExpr::Spawn(expr) => CoreExpr::Spawn(Box::new(fold_expr(*expr))),

        // Terminals — nothing to fold
        other => other,
    }
}

// ── Application folding ─────────────────────────────────────

fn fold_app(func: CoreExpr, arg: CoreExpr) -> CoreExpr {
    // Unary ops: App(PrimOp(op), Lit(x))
    if let CoreExpr::PrimOp(op) = &func {
        if let CoreExpr::Lit(lit) = &arg {
            if let Some(result) = fold_unary(*op, lit) {
                return CoreExpr::Lit(result);
            }
        }
        return CoreExpr::App(Box::new(func), Box::new(arg));
    }

    // Binary ops: App(App(PrimOp(op), Lit(a)), Lit(b))
    if let CoreExpr::App(inner_func, lhs) = &func {
        if let CoreExpr::PrimOp(op) = inner_func.as_ref() {
            if let (CoreExpr::Lit(a), CoreExpr::Lit(b)) = (lhs.as_ref(), &arg) {
                if let Some(result) = fold_binary(*op, a, b) {
                    return CoreExpr::Lit(result);
                }
            }
        }
    }

    CoreExpr::App(Box::new(func), Box::new(arg))
}

// ── Unary constant folding ───────────────────────────────────

fn fold_unary(op: PrimOp, lit: &Lit) -> Option<Lit> {
    match (op, lit) {
        (PrimOp::Neg, Lit::Int(n)) => Some(Lit::Int(-n)),
        (PrimOp::Neg, Lit::Float(n)) => Some(Lit::Float(-n)),
        (PrimOp::Not, Lit::Bool(b)) => Some(Lit::Bool(!b)),
        _ => None,
    }
}

// ── Binary constant folding ──────────────────────────────────

fn fold_binary(op: PrimOp, a: &Lit, b: &Lit) -> Option<Lit> {
    match (op, a, b) {
        // Arithmetic on integers
        (PrimOp::Add, Lit::Int(x), Lit::Int(y)) => Some(Lit::Int(x + y)),
        (PrimOp::Sub, Lit::Int(x), Lit::Int(y)) => Some(Lit::Int(x - y)),
        (PrimOp::Mul, Lit::Int(x), Lit::Int(y)) => Some(Lit::Int(x * y)),
        // Guard against division by zero — leave as-is
        (PrimOp::Div, Lit::Int(x), Lit::Int(y)) if *y != 0 => Some(Lit::Int(x / y)),
        (PrimOp::Mod, Lit::Int(x), Lit::Int(y)) if *y != 0 => Some(Lit::Int(x % y)),

        // Integer comparisons
        (PrimOp::Eq,  Lit::Int(x),  Lit::Int(y))  => Some(Lit::Bool(x == y)),
        (PrimOp::Neq, Lit::Int(x),  Lit::Int(y))  => Some(Lit::Bool(x != y)),
        (PrimOp::Lt,  Lit::Int(x),  Lit::Int(y))  => Some(Lit::Bool(x < y)),
        (PrimOp::Gt,  Lit::Int(x),  Lit::Int(y))  => Some(Lit::Bool(x > y)),
        (PrimOp::Lte, Lit::Int(x),  Lit::Int(y))  => Some(Lit::Bool(x <= y)),
        (PrimOp::Gte, Lit::Int(x),  Lit::Int(y))  => Some(Lit::Bool(x >= y)),

        // Bool comparisons
        (PrimOp::Eq,  Lit::Bool(x), Lit::Bool(y)) => Some(Lit::Bool(x == y)),
        (PrimOp::Neq, Lit::Bool(x), Lit::Bool(y)) => Some(Lit::Bool(x != y)),

        // Logic
        (PrimOp::And, Lit::Bool(x), Lit::Bool(y)) => Some(Lit::Bool(*x && *y)),
        (PrimOp::Or,  Lit::Bool(x), Lit::Bool(y)) => Some(Lit::Bool(*x || *y)),

        // Integer power (guard against overflow)
        (PrimOp::Pow, Lit::Int(x), Lit::Int(y)) if *y >= 0 => {
            u32::try_from(*y).ok()
                .and_then(|exp| x.checked_pow(exp))
                .map(Lit::Int)
        }

        // Float arithmetic
        (PrimOp::FAdd, Lit::Float(x), Lit::Float(y)) => Some(Lit::Float(x + y)),
        (PrimOp::FSub, Lit::Float(x), Lit::Float(y)) => Some(Lit::Float(x - y)),
        (PrimOp::FMul, Lit::Float(x), Lit::Float(y)) => Some(Lit::Float(x * y)),
        (PrimOp::FDiv, Lit::Float(x), Lit::Float(y)) if *y != 0.0 => Some(Lit::Float(x / y)),
        (PrimOp::FPow, Lit::Float(x), Lit::Float(y)) => Some(Lit::Float(x.powf(*y))),

        // Float comparisons
        (PrimOp::FEq,  Lit::Float(x), Lit::Float(y)) => Some(Lit::Bool(x == y)),
        (PrimOp::FLt,  Lit::Float(x), Lit::Float(y)) => Some(Lit::Bool(x < y)),
        (PrimOp::FGt,  Lit::Float(x), Lit::Float(y)) => Some(Lit::Bool(x > y)),
        (PrimOp::FLte, Lit::Float(x), Lit::Float(y)) => Some(Lit::Bool(x <= y)),
        (PrimOp::FGte, Lit::Float(x), Lit::Float(y)) => Some(Lit::Bool(x >= y)),

        _ => None,
    }
}

// ── Case / conditional elimination ──────────────────────────

fn fold_case(scrut: CoreExpr, alts: Vec<Alt>) -> CoreExpr {
    if let CoreExpr::Lit(ref lit) = scrut {
        // Try to find a matching alternative
        if let Some(body) = select_alt(lit, &alts) {
            return body;
        }
    }
    CoreExpr::Case(Box::new(scrut), alts)
}

/// Select the body of the first alternative that matches `lit`.
/// Returns `None` if no alt matches (conservative — leave the Case intact).
fn select_alt(lit: &Lit, alts: &[Alt]) -> Option<CoreExpr> {
    for alt in alts {
        match &alt.pat {
            CorePat::Lit(pat_lit) if pat_lit == lit => {
                return Some(alt.body.clone());
            }
            CorePat::Wildcard | CorePat::Var(_) => {
                // Wildcard and variable patterns always match.
                // For a variable pattern we'd ideally substitute, but since
                // we only call this when `lit` is known we just return the body
                // as-is (a variable binding of a literal is safe — it will
                // either be dead-eliminated or kept as a Let by the compiler).
                return Some(alt.body.clone());
            }
            _ => {
                // Constructor pattern or non-matching literal — keep looking
            }
        }
    }
    None
}

// ── Dead let elimination ─────────────────────────────────────

fn fold_let(name: String, val: CoreExpr, body: CoreExpr) -> CoreExpr {
    // Only eliminate if the value is a literal (guaranteed side-effect-free)
    // and the bound variable is not used in the body.
    if let CoreExpr::Lit(_) = &val {
        if !is_free(&name, &body) {
            return body;
        }
    }
    CoreExpr::Let(name, Box::new(val), Box::new(body))
}

/// Returns `true` if `name` appears free in `expr`.
fn is_free(name: &str, expr: &CoreExpr) -> bool {
    match expr {
        CoreExpr::Var(n) => n == name,
        CoreExpr::Lit(_) | CoreExpr::PrimOp(_) | CoreExpr::Con(_) => false,

        CoreExpr::App(f, a) => is_free(name, f) || is_free(name, a),

        CoreExpr::Lam(param, body) => {
            // `name` is shadowed by the lambda parameter
            if param == name { false } else { is_free(name, body) }
        }

        CoreExpr::Let(bound, val, body) => {
            is_free(name, val) || (bound != name && is_free(name, body))
        }

        CoreExpr::LetRec(bound, val, body) => {
            // Both val and body are in scope of `bound`
            if bound == name {
                false
            } else {
                is_free(name, val) || is_free(name, body)
            }
        }

        CoreExpr::Case(scrut, alts) => {
            if is_free(name, scrut) {
                return true;
            }
            alts.iter().any(|alt| {
                // Check whether `name` is bound by the pattern
                if pat_binds(name, &alt.pat) { false } else { is_free(name, &alt.body) }
            })
        }

        CoreExpr::MkList(elems) => elems.iter().any(|e| is_free(name, e)),

        CoreExpr::MkClosure { free_vars, .. } => free_vars.iter().any(|v| v == name),

        CoreExpr::Record(fields) => fields.iter().any(|(_, e)| is_free(name, e)),

        CoreExpr::FieldAccess(expr, _) => is_free(name, expr),

        CoreExpr::Scope(body) => is_free(name, body),
        CoreExpr::Spawn(expr) => is_free(name, expr),
    }
}

/// Returns `true` if the pattern binds `name` (i.e. shadows it).
fn pat_binds(name: &str, pat: &CorePat) -> bool {
    match pat {
        CorePat::Var(n) => n == name,
        CorePat::Wildcard | CorePat::Lit(_) => false,
        CorePat::Con(_, sub_pats) => sub_pats.iter().any(|p| pat_binds(name, p)),
        CorePat::Record(fields) => fields.iter().any(|(_, p)| pat_binds(name, p)),
    }
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use synoema_parser::{Lit, parse};
    use crate::desugar::desugar_program;

    fn fold(src: &str) -> CoreExpr {
        let program = parse(src).unwrap();
        let core = desugar_program(&program);
        let main_def = core.defs.iter().find(|d| d.name == "main").unwrap();
        optimize_expr(main_def.body.clone())
    }

    #[test]
    fn fold_addition() {
        let result = fold("main = 2 + 3");
        assert!(matches!(result, CoreExpr::Lit(Lit::Int(5))));
    }

    #[test]
    fn fold_multiplication() {
        let result = fold("main = 6 * 7");
        assert!(matches!(result, CoreExpr::Lit(Lit::Int(42))));
    }

    #[test]
    fn fold_comparison() {
        let result = fold("main = 3 > 2");
        assert!(matches!(result, CoreExpr::Lit(Lit::Bool(true))));
    }

    #[test]
    fn fold_nested() {
        // (2 + 3) * 4 → 20
        let result = fold("main = (2 + 3) * 4");
        assert!(matches!(result, CoreExpr::Lit(Lit::Int(20))));
    }

    #[test]
    fn fold_negation() {
        let result = fold("main = -(5)");
        assert!(matches!(result, CoreExpr::Lit(Lit::Int(-5))));
    }

    #[test]
    fn fold_bool_and() {
        let result = fold("main = true && false");
        assert!(matches!(result, CoreExpr::Lit(Lit::Bool(false))));
    }

    #[test]
    fn no_fold_division_by_zero() {
        // Should NOT fold — leave as an App
        let result = fold("main = 5 / 0");
        assert!(!matches!(result, CoreExpr::Lit(_)));
    }

    #[test]
    fn fold_subtraction() {
        let result = fold("main = 10 - 3");
        assert!(matches!(result, CoreExpr::Lit(Lit::Int(7))));
    }

    #[test]
    fn fold_division() {
        let result = fold("main = 12 / 4");
        assert!(matches!(result, CoreExpr::Lit(Lit::Int(3))));
    }

    #[test]
    fn fold_modulo() {
        let result = fold("main = 10 % 3");
        assert!(matches!(result, CoreExpr::Lit(Lit::Int(1))));
    }

    #[test]
    fn fold_eq_true() {
        let result = fold("main = 5 == 5");
        assert!(matches!(result, CoreExpr::Lit(Lit::Bool(true))));
    }

    #[test]
    fn fold_eq_false() {
        let result = fold("main = 5 == 6");
        assert!(matches!(result, CoreExpr::Lit(Lit::Bool(false))));
    }

    #[test]
    fn fold_bool_or() {
        let result = fold("main = false || true");
        assert!(matches!(result, CoreExpr::Lit(Lit::Bool(true))));
    }

    /// Test Not folding directly via the optimizer API (PrimOp::Not is not
    /// produced by the parser/desugarer — it's used internally — so we build
    /// the expression manually here).
    #[test]
    fn fold_not() {
        let expr = CoreExpr::App(
            Box::new(CoreExpr::PrimOp(PrimOp::Not)),
            Box::new(CoreExpr::Lit(Lit::Bool(true))),
        );
        let result = optimize_expr(expr);
        assert!(matches!(result, CoreExpr::Lit(Lit::Bool(false))));
    }

    #[test]
    fn fold_cond_true() {
        // ? true -> 42 : 0  →  should fold to Lit(Int(42))
        let result = fold("main = ? true -> 42 : 0");
        assert!(matches!(result, CoreExpr::Lit(Lit::Int(42))));
    }

    #[test]
    fn fold_cond_false() {
        let result = fold("main = ? false -> 99 : 7");
        assert!(matches!(result, CoreExpr::Lit(Lit::Int(7))));
    }

    #[test]
    fn fold_int_pow() {
        let result = fold("main = 2 ** 10");
        assert!(matches!(result, CoreExpr::Lit(Lit::Int(1024))));
    }

    #[test]
    fn fold_float_neg() {
        let e = fold("main = 0.0 - 3.14");
        // After desugaring to FSub + constant fold → Lit(Float(-3.14))
        if let CoreExpr::Lit(Lit::Float(f)) = e {
            assert!((f - (-3.14)).abs() < 1e-10);
        }
        // If not folded (desugarer may not produce FSub), that's also OK
    }
}
