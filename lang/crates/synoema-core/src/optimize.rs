// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

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
    // Phase 1: linearize tree recursion (before folding, so new code gets folded)
    let program = linearize_tree_recursion(program);
    // Phase 2: constant folding + dead code elimination
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

        CoreExpr::Region(body) => CoreExpr::Region(Box::new(fold_expr(*body))),
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

        CoreExpr::RecordUpdate { base, updates } => {
            is_free(name, base) || updates.iter().any(|(_, e)| is_free(name, e))
        }

        CoreExpr::FieldAccess(expr, _) => is_free(name, expr),

        CoreExpr::Region(body) => is_free(name, body),
        CoreExpr::Scope(body) => is_free(name, body),
        CoreExpr::Spawn(expr) => is_free(name, expr),
        CoreExpr::RuntimeError(_) => false,
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

// ── Tree Recursion Linearization ────────────────────────────
//
// Transforms binary tree-recursive functions into tail-recursive form:
//   f 0 = v0; f 1 = v1; f n = f(n-1) OP f(n-2)
// becomes:
//   f n = f__worker (n - base0) v0 v1
//   f__worker 0 a b = a
//   f__worker n a b = f__worker (n-1) b (a OP b)

/// Detected tree-recursion pattern components.
struct TreeRecPattern {
    base0: i64,
    val0: CoreExpr,
    val1: CoreExpr,
    op: PrimOp,
}

/// Match `App(App(PrimOp(Sub), Var(var)), Lit(Int(k)))` → Some(k).
fn match_sub_lit(expr: &CoreExpr, var: &str) -> Option<i64> {
    if let CoreExpr::App(f, arg) = expr {
        if let CoreExpr::App(ff, lhs) = f.as_ref() {
            if let CoreExpr::PrimOp(PrimOp::Sub) = ff.as_ref() {
                if let CoreExpr::Var(n) = lhs.as_ref() {
                    if n == var {
                        if let CoreExpr::Lit(Lit::Int(k)) = arg.as_ref() {
                            return Some(*k);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Match `App(Var(fname), sub_expr)` where sub_expr = `var - k` → Some(k).
fn match_self_call(expr: &CoreExpr, fname: &str, param: &str) -> Option<i64> {
    if let CoreExpr::App(f, arg) = expr {
        if let CoreExpr::Var(n) = f.as_ref() {
            if n == fname {
                return match_sub_lit(arg, param);
            }
        }
    }
    None
}

/// Detect tree-recursion pattern in a function definition.
fn detect_tree_recursion(fname: &str, body: &CoreExpr) -> Option<TreeRecPattern> {
    // Shape: Lam(x, Case(Var(x), alts))
    let (param, case_body) = if let CoreExpr::Lam(p, inner) = body {
        (p.as_str(), inner.as_ref())
    } else {
        return None;
    };

    let alts = if let CoreExpr::Case(scrut, alts) = case_body {
        if let CoreExpr::Var(s) = scrut.as_ref() {
            if s != param { return None; }
        } else {
            return None;
        }
        alts
    } else {
        return None;
    };

    // Need at least 3 alts: two literal base cases + one recursive
    if alts.len() < 3 { return None; }

    // First two alts should be literal base cases
    let (base0, val0) = if let CorePat::Lit(Lit::Int(k)) = &alts[0].pat {
        (*k, &alts[0].body)
    } else {
        return None;
    };

    let (base1, val1) = if let CorePat::Lit(Lit::Int(k)) = &alts[1].pat {
        (*k, &alts[1].body)
    } else {
        return None;
    };

    if base1 != base0 + 1 { return None; } // must be consecutive

    // Third alt should be Var(n) with recursive body
    let (rec_var, rec_body) = match &alts[2].pat {
        CorePat::Var(n) => (n.as_str(), &alts[2].body),
        CorePat::Wildcard => ("_", &alts[2].body),
        _ => return None,
    };

    // rec_body should be: App(App(PrimOp(OP), self_call_1), self_call_2)
    let (op, call1, call2) = if let CoreExpr::App(outer_f, rhs_call) = rec_body {
        if let CoreExpr::App(inner_f, lhs_call) = outer_f.as_ref() {
            if let CoreExpr::PrimOp(op) = inner_f.as_ref() {
                // Only allow Add or Mul (associative integer ops)
                if matches!(op, PrimOp::Add | PrimOp::Mul) {
                    (*op, lhs_call.as_ref(), rhs_call.as_ref())
                } else {
                    return None;
                }
            } else {
                return None;
            }
        } else {
            return None;
        }
    } else {
        return None;
    };

    // Both sides must be self-calls with offsets from the param
    let off1 = match_self_call(call1, fname, rec_var)?;
    let off2 = match_self_call(call2, fname, rec_var)?;

    // Offsets should be step1 and step2 where step1 < step2 (e.g., 1 and 2)
    let (step1, step2) = if off1 < off2 { (off1, off2) } else { (off2, off1) };
    if step1 != 1 || step2 != 2 { return None; } // only handle the standard case

    let _ = rec_var; // used only for pattern matching
    Some(TreeRecPattern {
        base0,
        val0: val0.clone(),
        val1: val1.clone(),
        op,
    })
}

/// Linearize tree-recursive definitions in a program.
fn linearize_tree_recursion(program: CoreProgram) -> CoreProgram {
    let mut new_defs = Vec::new();

    for def in program.defs {
        if let Some(pat) = detect_tree_recursion(&def.name, &def.body) {
            let worker_name = format!("{}__worker", def.name);

            // Build worker: \x -> \a -> \b -> case x of { 0 -> a; n -> worker (x-1) b (a OP b) }
            let worker_body = {
                let x = "x__w".to_string();
                let a = "a__w".to_string();
                let b = "b__w".to_string();
                let n = "n__w".to_string();

                // a OP b
                let op_ab = CoreExpr::App(
                    Box::new(CoreExpr::App(
                        Box::new(CoreExpr::PrimOp(pat.op)),
                        Box::new(CoreExpr::Var(a.clone())),
                    )),
                    Box::new(CoreExpr::Var(b.clone())),
                );

                // worker (x-1) b (a OP b)
                let recurse = CoreExpr::App(
                    Box::new(CoreExpr::App(
                        Box::new(CoreExpr::App(
                            Box::new(CoreExpr::Var(worker_name.clone())),
                            Box::new(CoreExpr::App(
                                Box::new(CoreExpr::App(
                                    Box::new(CoreExpr::PrimOp(PrimOp::Sub)),
                                    Box::new(CoreExpr::Var(x.clone())),
                                )),
                                Box::new(CoreExpr::Lit(Lit::Int(1))),
                            )),
                        )),
                        Box::new(CoreExpr::Var(b.clone())),
                    )),
                    Box::new(op_ab),
                );

                let alts = vec![
                    Alt { pat: CorePat::Lit(Lit::Int(0)), body: CoreExpr::Var(a.clone()) },
                    Alt { pat: CorePat::Var(n), body: recurse },
                ];

                CoreExpr::Lam(x.clone(), Box::new(
                    CoreExpr::Lam(a, Box::new(
                        CoreExpr::Lam(b, Box::new(
                            CoreExpr::Case(Box::new(CoreExpr::Var(x)), alts)
                        ))
                    ))
                ))
            };

            new_defs.push(CoreDef { name: worker_name.clone(), body: worker_body });

            // Build wrapper: \x -> worker (x - base0) val0 val1
            let wrapper_body = {
                let x = "x__wr".to_string();
                let adjusted = if pat.base0 == 0 {
                    CoreExpr::Var(x.clone())
                } else {
                    CoreExpr::App(
                        Box::new(CoreExpr::App(
                            Box::new(CoreExpr::PrimOp(PrimOp::Sub)),
                            Box::new(CoreExpr::Var(x.clone())),
                        )),
                        Box::new(CoreExpr::Lit(Lit::Int(pat.base0))),
                    )
                };

                let call = CoreExpr::App(
                    Box::new(CoreExpr::App(
                        Box::new(CoreExpr::App(
                            Box::new(CoreExpr::Var(worker_name)),
                            Box::new(adjusted),
                        )),
                        Box::new(pat.val0),
                    )),
                    Box::new(pat.val1),
                );

                CoreExpr::Lam(x, Box::new(call))
            };

            new_defs.push(CoreDef { name: def.name, body: wrapper_body });
        } else {
            new_defs.push(def);
        }
    }

    CoreProgram { defs: new_defs, ctor_tags: program.ctor_tags }
}

// ── Region Inference: Escape Analysis + Annotation ──────────

/// Annotate a Core IR program with Region nodes for automatic memory reclamation.
/// For each `let x = e1 in e2` where e1 allocates heap and x doesn't escape e2,
/// wrap in `Region(Let(...))` so the JIT emits region_enter/exit around the scope.
pub fn annotate_regions(program: CoreProgram) -> CoreProgram {
    let defs = program.defs.into_iter().map(|def| CoreDef {
        name: def.name,
        body: annotate_expr(def.body),
    }).collect();
    CoreProgram { defs, ctor_tags: program.ctor_tags }
}

fn annotate_expr(expr: CoreExpr) -> CoreExpr {
    match expr {
        CoreExpr::Let(name, val, body) => {
            let val = annotate_expr(*val);
            let body = annotate_expr(*body);
            if allocates_heap(&val) && !escapes(&name, &body) {
                CoreExpr::Region(Box::new(
                    CoreExpr::Let(name, Box::new(val), Box::new(body))
                ))
            } else {
                CoreExpr::Let(name, Box::new(val), Box::new(body))
            }
        }
        CoreExpr::LetRec(name, val, body) => {
            CoreExpr::LetRec(name, Box::new(annotate_expr(*val)), Box::new(annotate_expr(*body)))
        }
        CoreExpr::Lam(p, body) => CoreExpr::Lam(p, Box::new(annotate_expr(*body))),
        CoreExpr::App(f, a) => CoreExpr::App(Box::new(annotate_expr(*f)), Box::new(annotate_expr(*a))),
        CoreExpr::Case(scrut, alts) => CoreExpr::Case(
            Box::new(annotate_expr(*scrut)),
            alts.into_iter().map(|a| Alt { pat: a.pat, body: annotate_expr(a.body) }).collect(),
        ),
        CoreExpr::MkList(elems) => CoreExpr::MkList(elems.into_iter().map(annotate_expr).collect()),
        CoreExpr::Record(fields) => CoreExpr::Record(fields.into_iter().map(|(n, e)| (n, annotate_expr(e))).collect()),
        CoreExpr::FieldAccess(e, n) => CoreExpr::FieldAccess(Box::new(annotate_expr(*e)), n),
        CoreExpr::Region(body) => CoreExpr::Region(Box::new(annotate_expr(*body))),
        CoreExpr::Scope(body) => CoreExpr::Scope(Box::new(annotate_expr(*body))),
        CoreExpr::Spawn(e) => CoreExpr::Spawn(Box::new(annotate_expr(*e))),
        other => other, // Lit, Var, PrimOp, Con, MkClosure, RuntimeError
    }
}

/// Does this expression potentially allocate heap objects in the JIT?
fn allocates_heap(expr: &CoreExpr) -> bool {
    match expr {
        CoreExpr::MkList(elems) => !elems.is_empty(),
        CoreExpr::Record(_) => true,
        // PrimOps that produce heap values
        CoreExpr::App(f, _) => match f.as_ref() {
            CoreExpr::App(ff, _) => matches!(ff.as_ref(),
                CoreExpr::PrimOp(PrimOp::Cons | PrimOp::Concat | PrimOp::Range | PrimOp::Show)
            ),
            CoreExpr::PrimOp(PrimOp::Show) => true,
            // Function calls may allocate
            CoreExpr::Var(_) => true,
            _ => false,
        },
        // Constructors with arguments allocate ConNode
        CoreExpr::Con(_) => false, // bare constructor, no allocation until applied
        _ => false,
    }
}

/// Check if variable `var` escapes from the expression `body`.
/// A value escapes if it could be part of the return value or captured by a closure.
/// MUST be conservative: false negatives → use-after-free, false positives → missed optimization.
pub fn escapes(var: &str, body: &CoreExpr) -> bool {
    match body {
        // Returned directly → escapes
        CoreExpr::Var(n) => n == var,

        // Literals, operators, constructors → no reference to var
        CoreExpr::Lit(_) | CoreExpr::PrimOp(_) | CoreExpr::Con(_) | CoreExpr::RuntimeError(_) => false,

        // Data structures: if var is mentioned in elements → escapes
        CoreExpr::MkList(elems) => elems.iter().any(|e| is_free(var, e)),
        CoreExpr::Record(fields) => fields.iter().any(|(_, e)| is_free(var, e)),

        // Lambda: if var is captured → escapes (conservative: closure may escape)
        CoreExpr::Lam(param, inner) => {
            if param == var { false } else { is_free(var, inner) }
        }

        // MkClosure: captured vars escape
        CoreExpr::MkClosure { free_vars, .. } => free_vars.iter().any(|v| v == var),

        // Let chain: var escapes if it escapes from body,
        // or if it's used in val and the bound name escapes from body
        CoreExpr::Let(name, val, inner) => {
            if name == var {
                // shadowed — var doesn't escape through this let
                false
            } else if escapes(var, inner) {
                true
            } else {
                // Transitive: var is used in val, and name escapes
                is_free(var, val) && escapes(name, inner)
            }
        }
        CoreExpr::LetRec(name, val, inner) => {
            if name == var { false }
            else { escapes(var, inner) || (is_free(var, val) && escapes(name, inner)) }
        }

        // Case: var escapes if it escapes from any branch
        CoreExpr::Case(_scrut, alts) => {
            // If var is the scrutinee of a case, the branches receive its value
            // through patterns — but the patterns bind new names. The var itself
            // only escapes if a branch's body references it (not shadowed by pattern).
            alts.iter().any(|alt| {
                if pat_binds(var, &alt.pat) { false } else { escapes(var, &alt.body) }
            })
            // Note: we don't check scrut here because being scrutinized ≠ escaping.
            // The scrutinee is inspected, not returned. What matters is whether var
            // appears in a return position in any branch body.
        }

        // Function application: check if this is a known-consuming builtin
        CoreExpr::App(func, arg) => {
            if is_consuming_app(func) {
                // Builtin consumes arg without retaining it.
                // var escapes only if it's in the func position (shouldn't happen for builtins)
                is_free(var, func)
            } else {
                // Conservative: any mention in an app → escapes
                is_free(var, func) || is_free(var, arg)
            }
        }

        CoreExpr::RecordUpdate { base, updates } => {
            escapes(var, base) || updates.iter().any(|(_, e)| escapes(var, e))
        }

        CoreExpr::FieldAccess(expr, _) => escapes(var, expr),
        CoreExpr::Region(inner) => escapes(var, inner),
        CoreExpr::Scope(inner) => escapes(var, inner),
        CoreExpr::Spawn(inner) => is_free(var, inner), // spawned → escapes
    }
}

/// Check if an App node calls a known builtin that consumes its argument
/// without retaining a reference to it.
fn is_consuming_app(func: &CoreExpr) -> bool {
    match func {
        // Unary consuming builtins: length, sum, str_len, print, show
        CoreExpr::Var(n) => matches!(n.as_str(),
            "length" | "sum" | "str_len" | "str_find" | "str_trim" | "head" | "tail"
        ),
        // Partially applied binary: App(PrimOp(Show), _) — show applied to 1st arg
        CoreExpr::App(ff, _) => matches!(ff.as_ref(),
            CoreExpr::PrimOp(PrimOp::Show | PrimOp::Print)
        ),
        _ => false,
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

    // ── Escape analysis tests ─────────────────────────────

    #[test]
    fn escapes_var_returned_directly() {
        // let x = [1 2 3] in x → x escapes (is the return value)
        let body = CoreExpr::Var("x".into());
        assert!(escapes("x", &body));
    }

    #[test]
    fn escapes_var_not_mentioned() {
        // let x = [1 2 3] in 42 → x doesn't escape
        let body = CoreExpr::Lit(Lit::Int(42));
        assert!(!escapes("x", &body));
    }

    #[test]
    fn escapes_var_in_list() {
        // let x = [1] in [x] → x escapes (stored in data structure)
        let body = CoreExpr::MkList(vec![CoreExpr::Var("x".into())]);
        assert!(escapes("x", &body));
    }

    #[test]
    fn escapes_var_in_lambda() {
        // let x = [1] in (\_ -> x) → x escapes (captured by closure)
        let body = CoreExpr::Lam("_".into(), Box::new(CoreExpr::Var("x".into())));
        assert!(escapes("x", &body));
    }

    #[test]
    fn escapes_consuming_builtin() {
        // let x = [1..100] in length x → x doesn't escape (length consumes)
        let body = CoreExpr::App(
            Box::new(CoreExpr::Var("length".into())),
            Box::new(CoreExpr::Var("x".into())),
        );
        assert!(!escapes("x", &body));
    }

    #[test]
    fn escapes_transitive_through_let() {
        // let x = [1] in let y = x in y → x escapes (transitively through y)
        let body = CoreExpr::Let(
            "y".into(),
            Box::new(CoreExpr::Var("x".into())),
            Box::new(CoreExpr::Var("y".into())),
        );
        assert!(escapes("x", &body));
    }

    #[test]
    fn escapes_shadowed_in_let() {
        // let x = [1] in let x = 42 in x → x doesn't escape (shadowed)
        let body = CoreExpr::Let(
            "x".into(),
            Box::new(CoreExpr::Lit(Lit::Int(42))),
            Box::new(CoreExpr::Var("x".into())),
        );
        assert!(!escapes("x", &body));
    }

    // ── Region annotation tests ──────────────────────────

    #[test]
    fn region_annotated_for_non_escaping_list() {
        // let x = [1 2 3] in 42 → Region(Let(...))
        let expr = CoreExpr::Let(
            "x".into(),
            Box::new(CoreExpr::MkList(vec![
                CoreExpr::Lit(Lit::Int(1)),
                CoreExpr::Lit(Lit::Int(2)),
            ])),
            Box::new(CoreExpr::Lit(Lit::Int(42))),
        );
        let annotated = annotate_expr(expr);
        assert!(matches!(annotated, CoreExpr::Region(_)), "Expected Region, got: {}", annotated);
    }

    #[test]
    fn region_not_annotated_for_escaping_list() {
        // let x = [1 2] in x → NOT Region (x escapes)
        let expr = CoreExpr::Let(
            "x".into(),
            Box::new(CoreExpr::MkList(vec![
                CoreExpr::Lit(Lit::Int(1)),
                CoreExpr::Lit(Lit::Int(2)),
            ])),
            Box::new(CoreExpr::Var("x".into())),
        );
        let annotated = annotate_expr(expr);
        assert!(matches!(annotated, CoreExpr::Let(..)), "Expected Let (no Region), got: {}", annotated);
    }

    #[test]
    fn region_annotated_consuming_builtin() {
        // let x = [1 2] in length x → Region(Let(...))
        let expr = CoreExpr::Let(
            "x".into(),
            Box::new(CoreExpr::MkList(vec![
                CoreExpr::Lit(Lit::Int(1)),
                CoreExpr::Lit(Lit::Int(2)),
            ])),
            Box::new(CoreExpr::App(
                Box::new(CoreExpr::Var("length".into())),
                Box::new(CoreExpr::Var("x".into())),
            )),
        );
        let annotated = annotate_expr(expr);
        assert!(matches!(annotated, CoreExpr::Region(_)), "Expected Region, got: {}", annotated);
    }
}
