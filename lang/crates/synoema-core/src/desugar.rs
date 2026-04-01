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

/// Desugar a complete program
pub fn desugar_program(program: &Program) -> CoreProgram {
    let mut fresh = Fresh::new();
    let mut defs = Vec::new();

    for decl in &program.decls {
        match decl {
            Decl::Func { name, equations, .. } => {
                let body = desugar_func(&mut fresh, equations);
                defs.push(CoreDef { name: name.clone(), body });
            }
            Decl::TypeSig(_) => {
                // Type signatures are used by the type checker, not needed in Core
            }
            Decl::TypeDef { .. } => {
                // ADT definitions: constructors are handled as Con values
            }
        }
    }

    CoreProgram { defs }
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
        }
    }
    body
}

/// Build guard: if all literal patterns match → body, else fallback
fn build_pattern_guard(
    fresh: &mut Fresh,
    eq: &Equation,
    args: &[Name],
    body: CoreExpr,
    fallback: CoreExpr,
) -> CoreExpr {
    let mut checks: Vec<(usize, &Lit)> = Vec::new();
    for (i, pat) in eq.pats.iter().enumerate() {
        if let Pat::Lit(lit) = pat {
            checks.push((i, lit));
        }
    }
    if checks.is_empty() { return body; }

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
            CoreExpr::App(
                Box::new(CoreExpr::App(
                    Box::new(CoreExpr::PrimOp(PrimOp::from_binop(op))),
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

        // ── Field access (future: records) ──────────
        ExprKind::Field(obj, field) => {
            // For now: desugar as function application of a "getter"
            CoreExpr::App(
                Box::new(CoreExpr::Var(format!("get_{}", field))),
                Box::new(desugar_expr(fresh, obj)),
            )
        }
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
    }
}
