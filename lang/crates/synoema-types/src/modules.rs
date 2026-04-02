//! Module resolution pass for Synoema.
//!
//! Converts `mod`/`use` declarations into regular top-level `Decl::Func` entries
//! so that the type checker and evaluator can work on a flat list of definitions.
//!
//! ## Transformation rules
//!
//! `mod Math\n  square x = x * x`
//! becomes:
//!   `Math.square x = x * x`   (qualified name)
//!
//! `use Math (square)` where `Math.square` has arity 1 becomes:
//!   `square a0 = Math.square a0`   (eta-expanded wrapper — works in both interpreter and JIT)
//!
//! `use Math (pi)` where `Math.pi` has arity 0 becomes:
//!   `pi = Math.pi`   (constant alias)

use synoema_lexer::Span;
use synoema_parser::{Decl, Equation, Expr, ExprKind, Pat, Program};

/// Flatten all `mod`/`use` declarations into regular `Decl::Func` items.
///
/// Returns a new `Program` whose `modules` and `uses` are empty and all
/// qualified definitions live in `decls`.
///
/// Ordering guarantee: module-body definitions and their aliases are placed
/// *before* the regular top-level declarations so that forward references
/// from `main` (which typically appears last) can see all imported names.
pub fn resolve_modules(program: Program) -> Program {
    let mut module_decls: Vec<Decl> = Vec::new();

    // 1. Expand each module: prefix every function name with "ModuleName."
    for module in program.modules {
        for decl in module.body {
            match decl {
                Decl::Func { name, equations, span } => {
                    let qualified = format!("{}.{}", module.name, name);
                    module_decls.push(Decl::Func {
                        name: qualified,
                        equations,
                        span,
                    });
                }
                // Type sigs and ADTs inside a module are kept as-is.
                other => module_decls.push(other),
            }
        }
    }

    // 2. Expand each `use` declaration with eta-expansion for functions.
    //    This ensures aliases work correctly in both the interpreter and JIT.
    let mut alias_decls: Vec<Decl> = Vec::new();
    for use_decl in program.uses {
        for name in use_decl.names {
            let qualified = format!("{}.{}", use_decl.module, name);
            let span = use_decl.span;

            // Find the arity of the original definition from module_decls
            let arity = module_decls.iter().find_map(|d| match d {
                Decl::Func { name: n, equations, .. } if n == &qualified => {
                    Some(equations.first().map(|eq| eq.pats.len()).unwrap_or(0))
                }
                _ => None,
            }).unwrap_or(0);

            if arity == 0 {
                // Constant alias: `name = Module.name`
                let body = Expr::new(ExprKind::Var(qualified), Span::dummy());
                let equation = Equation { pats: vec![], body, span };
                alias_decls.push(Decl::Func { name: name.clone(), equations: vec![equation], span });
            } else {
                // Eta-expanded wrapper: `name a0 a1 ... = Module.name a0 a1 ...`
                // This works for curried functions: applying one arg at a time.
                // We expand only the top-level arity (outer lambdas).
                let args: Vec<String> = (0..arity).map(|i| format!("__u{}", i)).collect();
                let arg_pats: Vec<Pat> = args.iter()
                    .map(|a| Pat::Var(a.clone()))
                    .collect();
                // Build body: Module.name a0 a1 ...
                let mut body = Expr::new(ExprKind::Var(qualified), Span::dummy());
                for arg in &args {
                    let arg_expr = Expr::new(ExprKind::Var(arg.clone()), Span::dummy());
                    body = Expr::new(ExprKind::App(Box::new(body), Box::new(arg_expr)), Span::dummy());
                }
                let equation = Equation { pats: arg_pats, body, span };
                alias_decls.push(Decl::Func { name: name.clone(), equations: vec![equation], span });
            }
        }
    }

    // Order: module bodies first, then aliases, then original top-level decls.
    let mut decls = module_decls;
    decls.extend(alias_decls);
    decls.extend(program.decls);

    Program {
        decls,
        modules: vec![],
        uses: vec![],
    }
}
