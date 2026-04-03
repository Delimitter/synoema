//! Desugaring: AST → Core IR
//!
//! Transformations performed:
//! 1. Multi-equation functions → single Lam + Case
//! 2. `? cond -> then : else` → Case on Bool
//! 3. `e1 |> e2` → App(e2, e1)
//! 4. `f >> g` → Lam(x, App(g, App(f, x)))
//! 5. `BinOp(op, l, r)` → App(App(PrimOp(op), l), r)
//! 6. `Block(bindings, body)` → nested Let
//! 7. `Neg(e)` → App(PrimOp(neg), e)
//! 8. `List [a, b, c]` → MkList [a, b, c]
//! 9. `Range(a, b)` → App(App(PrimOp(range), a), b)
//! 10. `ListComp` → concatMap/filter chain

use crate::core_ir::*;
use synoema_parser::*;
use std::collections::HashMap;

/// Fresh variable name generator
struct Fresh {
    counter: u32,
}

impl Fresh {
    fn new() -> Self { Fresh { counter: 0 } }

    fn gen(&mut self, prefix: &str) -> Name {
        self.counter += 1;
        format!("{}${}", prefix, self.counter)
    }
}

/// Strip parentheses from a pattern
fn strip_paren(pat: &Pat) -> &Pat {
    match pat {
        Pat::Paren(inner) => strip_paren(inner),
        other => other,
    }
}

/// Desugar a complete program
pub fn desugar_program(program: &Program) -> CoreProgram {
    let mut fresh = Fresh::new();
    let mut defs = Vec::new();
    let mut ctor_tags: HashMap<String, (i64, usize)> = HashMap::new();

    // Collect constructor tags from ADT definitions
    for decl in &program.decls {
        if let Decl::TypeDef { variants, .. } = decl {
            for (tag, variant) in variants.iter().enumerate() {
                ctor_tags.insert(variant.name.clone(), (tag as i64, variant.fields.len()));
            }
        }
    }

    // Collect impl method equations to prepend to existing functions (more-specific first)
    let mut impl_eqs: HashMap<String, Vec<Equation>> = HashMap::new();
    for decl in &program.decls {
        if let Decl::ImplDecl { methods, .. } = decl {
            for (method_name, equations) in methods {
                impl_eqs.entry(method_name.clone())
                    .or_default()
                    .extend(equations.iter().cloned());
            }
        }
    }

    for decl in &program.decls {
        match decl {
            Decl::Func { name, equations, .. } => {
                // Prepend impl equations so they take priority (more specific patterns)
                let prepend = impl_eqs.remove(name).unwrap_or_default();
                let mut all_eqs = prepend;
                all_eqs.extend(equations.iter().cloned());
                let body = desugar_func(&mut fresh, &all_eqs);
                defs.push(CoreDef { name: name.clone(), body });
            }
            Decl::TypeSig(_) => {
                // Type signatures are used by the type checker, not needed in Core
            }
            Decl::TypeDef { .. } => {
                // ADT definitions: constructors are handled as Con values
            }
            Decl::TraitDecl { .. } => {
                // Trait declarations: used by type checker only
            }
            Decl::ImplDecl { .. } => {
                // Impl methods already merged above via impl_eqs
            }
        }
    }

    // Register standalone impl methods not covered by any Func decl
    for (method_name, equations) in impl_eqs {
        let body = desugar_func(&mut fresh, &equations);
        defs.push(CoreDef { name: method_name, body });
    }

    CoreProgram { defs, ctor_tags }
}

/// Desugar a function (one or more equations) into a Core expression.
///
/// Single equation, no patterns: `f = expr` → just desugar expr
/// Single equation with patterns: `f x y = expr` → Lam(x, Lam(y, desugar(expr)))
/// Multiple equations: `f 0 = 1; f n = n * f (n-1)` →
///   Lam(arg$1, Case(arg$1, [0 -> 1, n -> n * f (n-1)]))
fn desugar_func(fresh: &mut Fresh, equations: &[Equation]) -> CoreExpr {
    if equations.len() == 1 && equations[0].pats.is_empty() {
        // Simple constant: `f = expr`
        return desugar_expr(fresh, &equations[0].body);
    }

    if equations.len() == 1 {
        // Single equation with patterns: desugar as nested lambdas
        let eq = &equations[0];
        let body = desugar_expr(fresh, &eq.body);
        return wrap_lambdas(fresh, &eq.pats, body);
    }

    // Multiple equations: determine arity from first equation
    let arity = equations[0].pats.len();

    if arity == 0 {
        // Multiple zero-arg equations — just use first (shouldn't happen normally)
        return desugar_expr(fresh, &equations[0].body);
    }

    if arity == 1 {
        // Single argument: generate Case directly
        let arg = fresh.gen("arg");
        let alts = equations.iter().map(|eq| {
            let pat = desugar_pattern(&eq.pats[0]);
            let body = desugar_expr(fresh, &eq.body);
            Alt { pat, body }
        }).collect();

        CoreExpr::Lam(
            arg.clone(),
            Box::new(CoreExpr::Case(Box::new(CoreExpr::Var(arg)), alts)),
        )
    } else {
        // Multi-argument: proper equation-chain matching
        let args: Vec<_> = (0..arity).map(|i| fresh.gen(&format!("arg{}", i))).collect();
        let result = build_equation_chain(fresh, equations, &args);
        let mut expr = result;
        for arg in args.into_iter().rev() {
            expr = CoreExpr::Lam(arg, Box::new(expr));
        }
        expr
    }
}

/// Build a chain that tries each equation in order.
fn build_equation_chain(
    fresh: &mut Fresh,
    equations: &[Equation],
    args: &[Name],
) -> CoreExpr {
    if equations.is_empty() {
        return CoreExpr::Lit(Lit::Int(0)); // non-exhaustive fallback
    }
    if equations.len() == 1 {
        return build_single_equation(fresh, &equations[0], args);
    }

    let eq = &equations[0];
    let rest = &equations[1..];
    let has_checks = eq.pats.iter().any(|p| !matches!(p, Pat::Var(_) | Pat::Wildcard));

    if !has_checks {
        // All variable/wildcard patterns — always matches
        build_single_equation(fresh, eq, args)
    } else {
        let body = build_single_equation(fresh, eq, args);
        let fallback = build_equation_chain(fresh, rest, args);
        build_pattern_guard(fresh, eq, args, body, fallback)
    }
}

/// Build code for a single equation: bind variable patterns, evaluate body
fn build_single_equation(
    fresh: &mut Fresh,
    eq: &Equation,
    args: &[Name],
) -> CoreExpr {
    let mut body = desugar_expr(fresh, &eq.body);
    for (i, pat) in eq.pats.iter().enumerate().rev() {
        match pat {
            Pat::Var(name) => {
                body = CoreExpr::Let(name.clone(), Box::new(CoreExpr::Var(args[i].clone())), Box::new(body));
            }
            Pat::Wildcard | Pat::Lit(_) => {} // wildcard: skip, lit: checked in guard
            Pat::Cons(head, tail) => {
                body = CoreExpr::Case(
                    Box::new(CoreExpr::Var(args[i].clone())),
                    vec![Alt {
                        pat: CorePat::Con("Cons".into(), vec![desugar_pattern(head), desugar_pattern(tail)]),
                        body,
                    }],
                );
            }
            Pat::Con(name, sub_pats) => {
                body = CoreExpr::Case(
                    Box::new(CoreExpr::Var(args[i].clone())),
                    vec![Alt {
                        pat: CorePat::Con(name.clone(), sub_pats.iter().map(desugar_pattern).collect()),
                        body,
                    }],
                );
            }
            Pat::Paren(inner) => {
                let fake_eq = Equation { pats: vec![inner.as_ref().clone()], body: eq.body.clone(), span: eq.span };
                body = build_single_equation(fresh, &fake_eq, &[args[i].clone()]);
            }
            Pat::Record(fields) => {
                body = CoreExpr::Case(
                    Box::new(CoreExpr::Var(args[i].clone())),
                    vec![Alt {
                        pat: CorePat::Record(fields.iter().map(|(name, p)| (name.clone(), desugar_pattern(p))).collect()),
                        body,
                    }],
                );
            }
        }
    }
    body
}

/// Inject a fallback expression as a wildcard arm into the innermost Case in body.
/// Used when a guard has Con/Cons/Record patterns (not Lit) — the Case already
/// handles matching, but needs a wildcard fallback for non-matching values.
fn inject_fallback(body: CoreExpr, fallback: CoreExpr) -> CoreExpr {
    match body {
        CoreExpr::Case(scrut, mut alts) => {
            // Add fallback as wildcard arm if not already there
            if !alts.iter().any(|a| matches!(a.pat, CorePat::Wildcard | CorePat::Var(_))) {
                alts.push(Alt { pat: CorePat::Wildcard, body: fallback });
            }
            CoreExpr::Case(scrut, alts)
        }
        CoreExpr::Let(name, val, inner) => {
            // Let wraps a Case — recurse into the body
            CoreExpr::Let(name, val, Box::new(inject_fallback(*inner, fallback)))
        }
        other => other, // cannot inject, return as-is
    }
}

/// Build guard: if all literal patterns match → body, else fallback
fn build_pattern_guard(
    _fresh: &mut Fresh,
    eq: &Equation,
    args: &[Name],
    body: CoreExpr,
    fallback: CoreExpr,
) -> CoreExpr {
    let mut checks: Vec<(usize, &Lit)> = Vec::new();
    for (i, pat) in eq.pats.iter().enumerate() {
        if let Pat::Lit(lit) = strip_paren(pat) {
            checks.push((i, lit));
        }
    }
    if checks.is_empty() {
        // No literal guards — Con/Cons/Record patterns are handled by Case in build_single_equation.
        // Inject the fallback as a wildcard arm so non-matching values fall through correctly.
        return inject_fallback(body, fallback);
    }

    // Build nested if-then-else for each literal check
    let mut result = body;
    for (arg_idx, lit) in checks.into_iter().rev() {
        let cond = CoreExpr::App(
            Box::new(CoreExpr::App(
                Box::new(CoreExpr::PrimOp(PrimOp::Eq)),
                Box::new(CoreExpr::Var(args[arg_idx].clone())),
            )),
            Box::new(CoreExpr::Lit(lit.clone())),
        );
        result = CoreExpr::Case(
            Box::new(cond),
            vec![
                Alt { pat: CorePat::Lit(Lit::Bool(true)), body: result },
                Alt { pat: CorePat::Lit(Lit::Bool(false)), body: fallback.clone() },
            ],
        );
    }
    // Bind variable patterns
    for (i, pat) in eq.pats.iter().enumerate().rev() {
        if let Pat::Var(name) = pat {
            result = CoreExpr::Let(name.clone(), Box::new(CoreExpr::Var(args[i].clone())), Box::new(result));
        }
    }
    result
}


/// Wrap a body expression in nested lambdas for pattern arguments.
/// For simple variable patterns: `Lam(x, body)`
/// For complex patterns: `Lam(arg$n, Case(arg$n, [pat -> body]))`
fn wrap_lambdas(fresh: &mut Fresh, pats: &[Pat], body: CoreExpr) -> CoreExpr {
    let mut result = body;
    for pat in pats.iter().rev() {
        match pat {
            Pat::Var(name) => {
                result = CoreExpr::Lam(name.clone(), Box::new(result));
            }
            Pat::Wildcard => {
                let arg = fresh.gen("_w");
                result = CoreExpr::Lam(arg, Box::new(result));
            }
            _ => {
                let arg = fresh.gen("arg");
                let core_pat = desugar_pattern(pat);
                result = CoreExpr::Lam(
                    arg.clone(),
                    Box::new(CoreExpr::Case(
                        Box::new(CoreExpr::Var(arg)),
                        vec![Alt { pat: core_pat, body: result }],
                    )),
                );
            }
        }
    }
    result
}

/// Desugar a surface expression into Core
fn desugar_expr(fresh: &mut Fresh, expr: &Expr) -> CoreExpr {
    match &expr.kind {
        // ── Direct translations ─────────────────────
        ExprKind::Lit(lit) => CoreExpr::Lit(lit.clone()),
        ExprKind::Var(name) => CoreExpr::Var(name.clone()),
        ExprKind::Con(name) => CoreExpr::Con(name.clone()),
        ExprKind::Paren(inner) => desugar_expr(fresh, inner),

        // ── Application ─────────────────────────────
        ExprKind::App(func, arg) => {
            CoreExpr::App(
                Box::new(desugar_expr(fresh, func)),
                Box::new(desugar_expr(fresh, arg)),
            )
        }

        // ── Lambda ──────────────────────────────────
        ExprKind::Lam(pats, body) => {
            let core_body = desugar_expr(fresh, body);
            wrap_lambdas(fresh, pats, core_body)
        }

        // ── Binary operators → PrimOp application ───
        ExprKind::BinOp(BinOp::Pipe, lhs, rhs) => {
            // e1 |> e2  →  App(e2, e1)
            CoreExpr::App(
                Box::new(desugar_expr(fresh, rhs)),
                Box::new(desugar_expr(fresh, lhs)),
            )
        }

        ExprKind::BinOp(BinOp::Seq, lhs, rhs) => {
            // a ; b  →  let _seq = a in b
            let tmp = fresh.gen("_seq");
            CoreExpr::Let(
                tmp,
                Box::new(desugar_expr(fresh, lhs)),
                Box::new(desugar_expr(fresh, rhs)),
            )
        }

        ExprKind::BinOp(BinOp::Compose, f, g) => {
            // f >> g  →  \x -> g (f x)
            let x = fresh.gen("x");
            CoreExpr::Lam(
                x.clone(),
                Box::new(CoreExpr::App(
                    Box::new(desugar_expr(fresh, g)),
                    Box::new(CoreExpr::App(
                        Box::new(desugar_expr(fresh, f)),
                        Box::new(CoreExpr::Var(x)),
                    )),
                )),
            )
        }

        ExprKind::BinOp(op, lhs, rhs) => {
            // op l r  →  App(App(PrimOp(op), l), r)
            // If either operand is a float literal, use float-specific PrimOp.
            let is_float_lit = |e: &Expr| matches!(&e.kind, ExprKind::Lit(Lit::Float(_)));
            let primop = if is_float_lit(lhs) || is_float_lit(rhs) {
                float_binop(op)
            } else {
                PrimOp::from_binop(op)
            };
            CoreExpr::App(
                Box::new(CoreExpr::App(
                    Box::new(CoreExpr::PrimOp(primop)),
                    Box::new(desugar_expr(fresh, lhs)),
                )),
                Box::new(desugar_expr(fresh, rhs)),
            )
        }

        // ── Unary minus ─────────────────────────────
        ExprKind::Neg(inner) => {
            CoreExpr::App(
                Box::new(CoreExpr::PrimOp(PrimOp::Neg)),
                Box::new(desugar_expr(fresh, inner)),
            )
        }

        // ── Conditional → Case on Bool ──────────────
        ExprKind::Cond(cond, then_e, else_e) => {
            CoreExpr::Case(
                Box::new(desugar_expr(fresh, cond)),
                vec![
                    Alt {
                        pat: CorePat::Lit(Lit::Bool(true)),
                        body: desugar_expr(fresh, then_e),
                    },
                    Alt {
                        pat: CorePat::Lit(Lit::Bool(false)),
                        body: desugar_expr(fresh, else_e),
                    },
                ],
            )
        }

        // ── List ────────────────────────────────────
        ExprKind::List(elems) => {
            CoreExpr::MkList(elems.iter().map(|e| desugar_expr(fresh, e)).collect())
        }

        // ── Range ───────────────────────────────────
        ExprKind::Range(from, to) => {
            CoreExpr::App(
                Box::new(CoreExpr::App(
                    Box::new(CoreExpr::PrimOp(PrimOp::Range)),
                    Box::new(desugar_expr(fresh, from)),
                )),
                Box::new(desugar_expr(fresh, to)),
            )
        }

        // ── List comprehension ──────────────────────
        ExprKind::ListComp(body, generators) => {
            desugar_list_comp(fresh, body, generators)
        }

        // ── Block (let bindings) ────────────────────
        ExprKind::Block(bindings, result) => {
            let mut core = desugar_expr(fresh, result);
            // Build nested lets from inside out
            for binding in bindings.iter().rev() {
                let val = desugar_expr(fresh, &binding.value);
                core = CoreExpr::Let(binding.name.clone(), Box::new(val), Box::new(core));
            }
            core
        }

        // ── Record literal ──────────────────────────
        ExprKind::Record(fields) => {
            CoreExpr::Record(
                fields.iter()
                    .map(|(name, expr)| (name.clone(), desugar_expr(fresh, expr)))
                    .collect()
            )
        }

        // ── Field access ─────────────────────────────
        ExprKind::Field(obj, field) => {
            CoreExpr::FieldAccess(Box::new(desugar_expr(fresh, obj)), field.clone())
        }

        // ── Concurrency (Phase BC) ───────────────────
        ExprKind::Scope(body) => CoreExpr::Scope(Box::new(desugar_expr(fresh, body))),
        ExprKind::Spawn(expr) => CoreExpr::Spawn(Box::new(desugar_expr(fresh, expr))),
    }
}

/// Desugar list comprehension:
/// `[body | x <- xs, guard, y <- ys]` →
/// `concatMap (\x -> filter (\_ -> guard) (concatMap (\y -> [body]) ys)) xs`
fn desugar_list_comp(
    fresh: &mut Fresh,
    body: &Expr,
    generators: &[Generator],
) -> CoreExpr {
    if generators.is_empty() {
        // Base case: [body] (singleton list)
        return CoreExpr::MkList(vec![desugar_expr(fresh, body)]);
    }

    match &generators[0] {
        Generator::Bind(var, source) => {
            // x <- xs, rest...  →  concatMap (\x -> desugar(rest)) xs
            let rest = desugar_list_comp(fresh, body, &generators[1..]);
            let lambda = CoreExpr::Lam(var.clone(), Box::new(rest));

            CoreExpr::App(
                Box::new(CoreExpr::App(
                    Box::new(CoreExpr::Var("concatMap".into())),
                    Box::new(lambda),
                )),
                Box::new(desugar_expr(fresh, source)),
            )
        }
        Generator::Guard(cond) => {
            // guard, rest...  →  case cond of True -> desugar(rest); False -> []
            let rest = desugar_list_comp(fresh, body, &generators[1..]);

            CoreExpr::Case(
                Box::new(desugar_expr(fresh, cond)),
                vec![
                    Alt {
                        pat: CorePat::Lit(Lit::Bool(true)),
                        body: rest,
                    },
                    Alt {
                        pat: CorePat::Lit(Lit::Bool(false)),
                        body: CoreExpr::MkList(vec![]),
                    },
                ],
            )
        }
    }
}

/// Map a BinOp to the corresponding float PrimOp.
/// Falls back to the integer PrimOp for operators that have no float variant
/// (e.g. comparison operators reuse the same PrimOp but dispatch at runtime).
fn float_binop(op: &BinOp) -> PrimOp {
    match op {
        BinOp::Add => PrimOp::FAdd,
        BinOp::Sub => PrimOp::FSub,
        BinOp::Mul => PrimOp::FMul,
        BinOp::Div => PrimOp::FDiv,
        BinOp::Pow => PrimOp::FPow,
        BinOp::Lt  => PrimOp::FLt,
        BinOp::Gt  => PrimOp::FGt,
        BinOp::Lte => PrimOp::FLte,
        BinOp::Gte => PrimOp::FGte,
        BinOp::Eq  => PrimOp::FEq,
        // All other operators fall back to integer PrimOp
        other => PrimOp::from_binop(other),
    }
}

/// Desugar a surface pattern into Core pattern
fn desugar_pattern(pat: &Pat) -> CorePat {
    match pat {
        Pat::Wildcard => CorePat::Wildcard,
        Pat::Var(name) => CorePat::Var(name.clone()),
        Pat::Lit(lit) => CorePat::Lit(lit.clone()),
        Pat::Con(name, sub_pats) => {
            CorePat::Con(
                name.clone(),
                sub_pats.iter().map(desugar_pattern).collect(),
            )
        }
        Pat::Cons(head, tail) => {
            CorePat::Con(
                "Cons".into(),
                vec![desugar_pattern(head), desugar_pattern(tail)],
            )
        }
        Pat::Paren(inner) => desugar_pattern(inner),
        Pat::Record(fields) => {
            CorePat::Record(
                fields.iter()
                    .map(|(name, pat)| (name.clone(), desugar_pattern(pat)))
                    .collect()
            )
        }
    }
}
