use crate::*;

// ── Helpers ───────────────────────────────────────────────

fn parse_ok(src: &str) -> Program {
    parse(src).unwrap_or_else(|e| panic!("Parse failed for {:?}: {}", src, e))
}

fn expr_ok(src: &str) -> Expr {
    parse_expr(src).unwrap_or_else(|e| panic!("Parse expr failed for {:?}: {}", src, e))
}

fn first_func(p: &Program) -> &Decl {
    p.decls.iter().find(|d| matches!(d, Decl::Func { .. })).expect("no func")
}

// ── Literals ──────────────────────────────────────────────

#[test]
fn expr_int() {
    let e = expr_ok("42");
    assert!(matches!(e.kind, ExprKind::Lit(Lit::Int(42))));
}

#[test]
fn expr_float() {
    let e = expr_ok("3.14");
    assert!(matches!(e.kind, ExprKind::Lit(Lit::Float(f)) if (f - 3.14).abs() < 1e-10));
}

#[test]
fn expr_string() {
    let e = expr_ok("\"hello\"");
    assert!(matches!(&e.kind, ExprKind::Lit(Lit::Str(s)) if s == "hello"));
}

#[test]
fn expr_bool() {
    let e = expr_ok("true");
    assert!(matches!(e.kind, ExprKind::Lit(Lit::Bool(true))));
}

// ── Variables & Constructors ──────────────────────────────

#[test]
fn expr_var() {
    let e = expr_ok("foo");
    assert!(matches!(&e.kind, ExprKind::Var(s) if s == "foo"));
}

#[test]
fn expr_constructor() {
    let e = expr_ok("Just");
    assert!(matches!(&e.kind, ExprKind::Con(s) if s == "Just"));
}

// ── Binary Operators (precedence) ─────────────────────────

#[test]
fn expr_add() {
    let e = expr_ok("1 + 2");
    assert!(matches!(&e.kind, ExprKind::BinOp(BinOp::Add, _, _)));
}

#[test]
fn expr_precedence_mul_over_add() {
    // 1 + 2 * 3 should parse as 1 + (2 * 3)
    let e = expr_ok("1 + 2 * 3");
    match &e.kind {
        ExprKind::BinOp(BinOp::Add, lhs, rhs) => {
            assert!(matches!(&lhs.kind, ExprKind::Lit(Lit::Int(1))));
            assert!(matches!(&rhs.kind, ExprKind::BinOp(BinOp::Mul, _, _)));
        }
        _ => panic!("Expected Add at top, got {:?}", e.kind),
    }
}

#[test]
fn expr_pipe_left_assoc() {
    // a |> f |> g  should parse as (a |> f) |> g
    let e = expr_ok("a |> f |> g");
    match &e.kind {
        ExprKind::BinOp(BinOp::Pipe, lhs, rhs) => {
            assert!(matches!(&lhs.kind, ExprKind::BinOp(BinOp::Pipe, _, _)));
            assert!(matches!(&rhs.kind, ExprKind::Var(s) if s == "g"));
        }
        _ => panic!("Expected Pipe at top"),
    }
}

#[test]
fn expr_concat() {
    let e = expr_ok("[1] ++ [2] ++ [3]");
    // ++ is right-associative: [1] ++ ([2] ++ [3])
    match &e.kind {
        ExprKind::BinOp(BinOp::Concat, _, rhs) => {
            assert!(matches!(&rhs.kind, ExprKind::BinOp(BinOp::Concat, _, _)));
        }
        _ => panic!("Expected Concat at top"),
    }
}

// ── Application ───────────────────────────────────────────

#[test]
fn expr_application() {
    // f x → App(f, x)
    let e = expr_ok("f x");
    assert!(matches!(&e.kind, ExprKind::App(_, _)));
}

#[test]
fn expr_app_higher_than_plus() {
    // f x + 1 → (f x) + 1
    let e = expr_ok("f x + 1");
    match &e.kind {
        ExprKind::BinOp(BinOp::Add, lhs, _) => {
            assert!(matches!(&lhs.kind, ExprKind::App(_, _)));
        }
        _ => panic!("Expected Add at top"),
    }
}

// ── Lambda ────────────────────────────────────────────────

#[test]
fn expr_lambda() {
    let e = expr_ok("\\x -> x + 1");
    match &e.kind {
        ExprKind::Lam(pats, body) => {
            assert_eq!(pats.len(), 1);
            assert!(matches!(&body.kind, ExprKind::BinOp(BinOp::Add, _, _)));
        }
        _ => panic!("Expected Lam"),
    }
}

#[test]
fn expr_lambda_multi_param() {
    let e = expr_ok("\\x y -> x + y");
    match &e.kind {
        ExprKind::Lam(pats, _) => assert_eq!(pats.len(), 2),
        _ => panic!("Expected Lam"),
    }
}

// ── Conditional ───────────────────────────────────────────

#[test]
fn expr_cond() {
    let e = expr_ok("? x > 0 -> x : 0");
    assert!(matches!(&e.kind, ExprKind::Cond(_, _, _)));
}

// ── Lists ─────────────────────────────────────────────────

#[test]
fn expr_empty_list() {
    let e = expr_ok("[]");
    assert!(matches!(&e.kind, ExprKind::List(v) if v.is_empty()));
}

#[test]
fn expr_list() {
    let e = expr_ok("[1 2 3]");
    match &e.kind {
        ExprKind::List(elems) => assert_eq!(elems.len(), 3),
        _ => panic!("Expected List"),
    }
}

#[test]
fn expr_range() {
    let e = expr_ok("[1..10]");
    assert!(matches!(&e.kind, ExprKind::Range(_, _)));
}

#[test]
fn expr_list_comp() {
    let e = expr_ok("[x | x <- xs , x > 0]");
    match &e.kind {
        ExprKind::ListComp(_, gens) => assert_eq!(gens.len(), 2),
        _ => panic!("Expected ListComp"),
    }
}

// ── Unary Minus ───────────────────────────────────────────

#[test]
fn expr_neg() {
    let e = expr_ok("-1");
    // Could be Neg(Int(1)) or Int(-1) depending on parser
    // Both are acceptable for now
    assert!(
        matches!(&e.kind, ExprKind::Neg(_)) ||
        matches!(&e.kind, ExprKind::Lit(Lit::Int(-1)))
    );
}

// ── Parentheses ───────────────────────────────────────────

#[test]
fn expr_parens() {
    let e = expr_ok("(1 + 2) * 3");
    match &e.kind {
        ExprKind::BinOp(BinOp::Mul, lhs, _) => {
            assert!(matches!(&lhs.kind, ExprKind::Paren(_)));
        }
        _ => panic!("Expected Mul at top"),
    }
}

// ── Function Definitions ─────────────────────────────────

#[test]
fn decl_simple_func() {
    let p = parse_ok("double x = x + x");
    match first_func(&p) {
        Decl::Func { name, equations, .. } => {
            assert_eq!(name, "double");
            assert_eq!(equations.len(), 1);
            assert_eq!(equations[0].pats.len(), 1);
        }
        _ => unreachable!(),
    }
}

#[test]
fn decl_pattern_match_func() {
    let p = parse_ok("fac 0 = 1\nfac n = n * fac (n - 1)");
    match first_func(&p) {
        Decl::Func { name, equations, .. } => {
            assert_eq!(name, "fac");
            assert_eq!(equations.len(), 2);
        }
        _ => unreachable!(),
    }
}

#[test]
fn decl_constant() {
    let p = parse_ok("pi = 3.14159");
    match first_func(&p) {
        Decl::Func { name, equations, .. } => {
            assert_eq!(name, "pi");
            assert_eq!(equations[0].pats.len(), 0);
        }
        _ => unreachable!(),
    }
}

// ── Type Signatures ───────────────────────────────────────

#[test]
fn decl_type_sig() {
    let p = parse_ok("add : Int -> Int -> Int");
    assert!(p.decls.iter().any(|d| matches!(d, Decl::TypeSig(ts) if ts.name == "add")));
}

// ── ADT Definitions ───────────────────────────────────────

#[test]
fn decl_adt_maybe() {
    let p = parse_ok("Maybe a = Just a | None");
    match &p.decls[0] {
        Decl::TypeDef { name, params, variants, .. } => {
            assert_eq!(name, "Maybe");
            assert_eq!(params, &["a"]);
            assert_eq!(variants.len(), 2);
            assert_eq!(variants[0].name, "Just");
            assert_eq!(variants[1].name, "None");
        }
        _ => panic!("Expected TypeDef"),
    }
}

#[test]
fn decl_adt_shape() {
    let p = parse_ok("Shape = Circle Float | Rect Float Float | Point");
    match &p.decls[0] {
        Decl::TypeDef { variants, .. } => {
            assert_eq!(variants.len(), 3);
            assert_eq!(variants[0].fields.len(), 1); // Circle Float
            assert_eq!(variants[1].fields.len(), 2); // Rect Float Float
            assert_eq!(variants[2].fields.len(), 0); // Point
        }
        _ => panic!("Expected TypeDef"),
    }
}

// ── Patterns ──────────────────────────────────────────────

#[test]
fn pattern_wildcard() {
    let p = parse_ok("f _ = 0");
    match first_func(&p) {
        Decl::Func { equations, .. } => {
            assert!(matches!(&equations[0].pats[0], Pat::Wildcard));
        }
        _ => unreachable!(),
    }
}

#[test]
fn pattern_constructor() {
    let p = parse_ok("f (Just x) = x\nf None = 0");
    match first_func(&p) {
        Decl::Func { equations, .. } => {
            assert_eq!(equations.len(), 2);
        }
        _ => unreachable!(),
    }
}

#[test]
fn pattern_cons() {
    let p = parse_ok("head (x:xs) = x");
    match first_func(&p) {
        Decl::Func { equations, .. } => {
            assert!(matches!(&equations[0].pats[0], Pat::Cons(_, _)));
        }
        _ => unreachable!(),
    }
}

#[test]
fn pattern_empty_list() {
    let p = parse_ok("len [] = 0");
    match first_func(&p) {
        Decl::Func { equations, .. } => {
            // [] pattern is Con("Nil", [])
            assert!(matches!(&equations[0].pats[0], Pat::Con(n, args) if n == "Nil" && args.is_empty()));
        }
        _ => unreachable!(),
    }
}

// ── Full Programs ─────────────────────────────────────────

#[test]
fn full_factorial() {
    let p = parse_ok("fac 0 = 1\nfac n = n * fac (n - 1)");
    assert!(!p.decls.is_empty());
}

#[test]
fn full_quicksort() {
    let src = "qsort [] = []\nqsort (p:xs) = qsort lo ++ [p] ++ qsort hi\n  lo = [x | x <- xs , x <= p]\n  hi = [x | x <- xs , x > p]";
    let p = parse_ok(src);
    assert!(!p.decls.is_empty());
}

#[test]
fn full_map() {
    let p = parse_ok("map f [] = []\nmap f (x:xs) = f x : map f xs");
    match first_func(&p) {
        Decl::Func { name, equations, .. } => {
            assert_eq!(name, "map");
            assert_eq!(equations.len(), 2);
        }
        _ => unreachable!(),
    }
}

// ── Error Cases ──────────────────────────────────────────

#[test]
fn error_missing_equals() {
    assert!(parse("f x 1").is_err());
}

#[test]
fn error_empty_input() {
    let p = parse("");
    assert!(p.is_ok());
    assert!(p.unwrap().decls.is_empty());
}
