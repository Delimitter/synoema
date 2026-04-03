// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

//! Derive synthesis: expand `deriving (Show, Eq, Ord)` into synthetic ImplDecl entries.
//!
//! Called after parsing, before type checking / eval / codegen.
//! Generated ImplDecl nodes are indistinguishable from hand-written ones.
//!
//! Key design constraint: the evaluator dispatches curried multi-arg functions
//! by committing to the first matching equation for the first argument. Therefore,
//! multi-arg derived methods (eq, cmp) use single-equation approaches with
//! nested conditionals that work correctly in both interpreter and JIT.

use crate::ast::*;
use synoema_lexer::Span;

/// Expand all `deriving` clauses in a program into synthetic `Decl::ImplDecl` entries.
/// Returns a list of errors for unknown derive traits.
pub fn expand_derives(program: &mut Program) -> Vec<DeriveError> {
    let mut errors = Vec::new();
    let mut synthetic = Vec::new();

    for decl in &program.decls {
        if let Decl::TypeDef { name, variants, derives, span, .. } = decl {
            if variants.is_empty() {
                continue;
            }
            for derive_name in derives {
                match derive_name.as_str() {
                    // Show is a no-op: the builtin show already handles all types including ADTs.
                    // We accept it syntactically for Haskell-like ergonomics.
                    "Show" => {}
                    "Eq" => synthetic.push(derive_eq(name, variants, *span)),
                    "Ord" => synthetic.push(derive_ord(name, variants, *span)),
                    _ => errors.push(DeriveError {
                        trait_name: derive_name.clone(),
                        type_name: name.clone(),
                        span: *span,
                    }),
                }
            }
        }
    }

    program.decls.extend(synthetic);
    errors
}

/// Error for unknown derive trait.
#[derive(Debug, Clone)]
pub struct DeriveError {
    pub trait_name: String,
    pub type_name: String,
    pub span: Span,
}

impl std::fmt::Display for DeriveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown derive trait '{}' for type '{}'", self.trait_name, self.type_name)
    }
}

// ── Helper: create AST nodes ────────────────────────────

fn dummy() -> Span { Span::dummy() }

fn var(name: &str) -> Expr {
    Expr::new(ExprKind::Var(name.to_string()), dummy())
}

fn int_lit(n: i64) -> Expr {
    Expr::new(ExprKind::Lit(Lit::Int(n)), dummy())
}

fn con(name: &str) -> Expr {
    Expr::new(ExprKind::Con(name.to_string()), dummy())
}

fn binop(op: BinOp, l: Expr, r: Expr) -> Expr {
    Expr::new(ExprKind::BinOp(op, Box::new(l), Box::new(r)), dummy())
}

fn pat_var(name: &str) -> Pat {
    Pat::Var(name.to_string())
}

fn equation(span: Span, pats: Vec<Pat>, body: Expr) -> Equation {
    Equation { pats, body, span }
}

fn cond(c: Expr, then_e: Expr, else_e: Expr) -> Expr {
    Expr::new(ExprKind::Cond(Box::new(c), Box::new(then_e), Box::new(else_e)), dummy())
}

fn block(bindings: Vec<Binding>, body: Expr) -> Expr {
    Expr::new(ExprKind::Block(bindings, Box::new(body)), dummy())
}

fn binding(name: &str, value: Expr) -> Binding {
    Binding { name: name.to_string(), value, span: dummy() }
}

// ── derive(Eq) ──────────────────────────────────────────

/// Generate: eq _x _y = _x == _y
///
/// The `==` operator already handles structural equality for all ADT types,
/// so derive(Eq) simply delegates to it.
fn derive_eq(type_name: &str, _variants: &[Variant], span: Span) -> Decl {
    let body = binop(BinOp::Eq, var("_x"), var("_y"));
    let equations = vec![equation(
        span,
        vec![pat_var("_x"), pat_var("_y")],
        body,
    )];

    Decl::ImplDecl {
        trait_name: "Eq".to_string(),
        ty_name: type_name.to_string(),
        methods: vec![("eq".to_string(), equations)],
        span,
    }
}

// ── derive(Ord) ─────────────────────────────────────────

/// Build a chain of conditionals to map a value to its variant index:
/// `? v == Red -> 0 : ? v == Green -> 1 : 2`
fn variant_index_chain(v: &str, variants: &[Variant]) -> Expr {
    let n = variants.len();
    if n == 0 { return int_lit(0); }
    if n == 1 { return int_lit(0); }

    // Build from right to left
    let mut result = int_lit((n - 1) as i64);
    for i in (0..n - 1).rev() {
        let check = binop(BinOp::Eq, var(v), con(&variants[i].name));
        result = cond(check, int_lit(i as i64), result);
    }
    result
}

/// Generate: cmp _x _y =
///   _xi = <variant_index_of _x>
///   _yi = <variant_index_of _y>
///   ? _xi == _yi -> 0 : ? _xi < _yi -> 0 - 1 : 1
///
/// For ADTs without fields, ordering is by declaration position.
/// For ADTs with fields in the same variant, returns 0 (no lexicographic
/// comparison of fields for now — can be extended later).
fn derive_ord(type_name: &str, variants: &[Variant], span: Span) -> Decl {
    let xi_expr = variant_index_chain("_x", variants);
    let yi_expr = variant_index_chain("_y", variants);

    let comparison = cond(
        binop(BinOp::Eq, var("_xi"), var("_yi")),
        int_lit(0),
        cond(
            binop(BinOp::Lt, var("_xi"), var("_yi")),
            binop(BinOp::Sub, int_lit(0), int_lit(1)),
            int_lit(1),
        ),
    );

    let body = block(
        vec![
            binding("_xi", xi_expr),
            binding("_yi", yi_expr),
        ],
        comparison,
    );

    let equations = vec![equation(
        span,
        vec![pat_var("_x"), pat_var("_y")],
        body,
    )];

    Decl::ImplDecl {
        trait_name: "Ord".to_string(),
        ty_name: type_name.to_string(),
        methods: vec![("cmp".to_string(), equations)],
        span,
    }
}
