// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

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

#[test]
fn pattern_singleton_list() {
    let p = parse_ok("f [x] = x");
    match first_func(&p) {
        Decl::Func { equations, .. } => {
            // [x] desugars to Cons(Var("x"), Con("Nil", []))
            assert!(matches!(&equations[0].pats[0], Pat::Cons(_, _)));
            if let Pat::Cons(head, tail) = &equations[0].pats[0] {
                assert!(matches!(head.as_ref(), Pat::Var(n) if n == "x"));
                assert!(matches!(tail.as_ref(), Pat::Con(n, args) if n == "Nil" && args.is_empty()));
            }
        }
        _ => unreachable!(),
    }
}

#[test]
fn pattern_multi_elem_list() {
    let p = parse_ok("f [x y z] = x");
    match first_func(&p) {
        Decl::Func { equations, .. } => {
            // [x y z] desugars to Cons(x, Cons(y, Cons(z, Nil)))
            if let Pat::Cons(head, tail) = &equations[0].pats[0] {
                assert!(matches!(head.as_ref(), Pat::Var(n) if n == "x"));
                if let Pat::Cons(head2, tail2) = tail.as_ref() {
                    assert!(matches!(head2.as_ref(), Pat::Var(n) if n == "y"));
                    if let Pat::Cons(head3, tail3) = tail2.as_ref() {
                        assert!(matches!(head3.as_ref(), Pat::Var(n) if n == "z"));
                        assert!(matches!(tail3.as_ref(), Pat::Con(n, args) if n == "Nil" && args.is_empty()));
                    } else {
                        panic!("expected Cons for z");
                    }
                } else {
                    panic!("expected Cons for y");
                }
            } else {
                panic!("expected Cons for [x y z]");
            }
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

#[test]
fn expr_record_literal() {
    let e = parse_expr("{name = \"Alice\", age = 30}").expect("parse failed");
    match e.kind {
        ExprKind::Record(fields) => {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "name");
            assert_eq!(fields[1].0, "age");
        }
        _ => panic!("Expected Record, got {:?}", e.kind),
    }
}

#[test]
fn expr_record_field_access() {
    let e = parse_expr("r.name").expect("parse failed");
    match e.kind {
        ExprKind::Field(_, field) => assert_eq!(field, "name"),
        _ => panic!("Expected Field, got {:?}", e.kind),
    }
}

#[test]
fn expr_record_empty() {
    let e = parse_expr("{}").expect("parse failed");
    match e.kind {
        ExprKind::Record(fields) => assert!(fields.is_empty()),
        _ => panic!("Expected empty Record"),
    }
}

#[test]
fn pattern_record() {
    let p = parse_ok("f {name = n, age = a} = n");
    match first_func(&p) {
        Decl::Func { equations, .. } => {
            assert_eq!(equations[0].pats.len(), 1);
            assert!(matches!(&equations[0].pats[0], Pat::Record(_)));
            if let Pat::Record(fields) = &equations[0].pats[0] {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "name");
                assert_eq!(fields[1].0, "age");
                assert!(matches!(&fields[0].1, Pat::Var(n) if n == "n"));
                assert!(matches!(&fields[1].1, Pat::Var(n) if n == "a"));
            }
        }
        _ => unreachable!(),
    }
}

#[test]
fn pattern_record_wildcard_field() {
    let p = parse_ok("f {x = _, y = n} = n");
    match first_func(&p) {
        Decl::Func { equations, .. } => {
            if let Pat::Record(fields) = &equations[0].pats[0] {
                assert!(matches!(&fields[0].1, Pat::Wildcard));
                assert!(matches!(&fields[1].1, Pat::Var(_)));
            } else {
                panic!("Expected record pattern");
            }
        }
        _ => unreachable!(),
    }
}

// ── Module Declarations ───────────────────────────────────

#[test]
fn parse_module_decl() {
    let src = "mod Math\n  square x = x * x\n\nuse Math (square)\n\nmain = square 5";
    let p = parse_ok(src);
    // One regular decl (main), one module, one use
    assert_eq!(p.modules.len(), 1);
    assert_eq!(p.modules[0].name, "Math");
    assert_eq!(p.modules[0].body.len(), 1);
    assert_eq!(p.uses.len(), 1);
    assert_eq!(p.uses[0].module, "Math");
    assert_eq!(p.uses[0].names, vec!["square"]);
    assert_eq!(p.decls.len(), 1);
}

#[test]
fn parse_module_multi_export() {
    let src = "mod Num\n  double x = x * 2\n  triple x = x * 3\n\nuse Num (double triple)\n\nmain = double 5";
    let p = parse_ok(src);
    assert_eq!(p.modules.len(), 1);
    assert_eq!(p.modules[0].body.len(), 2);
    assert_eq!(p.uses[0].names, vec!["double", "triple"]);
}

// ── Doc Comments ─────────────────────────────────────────

#[test]
fn doc_comment_on_func() {
    let p = parse_ok("--- Factorial function.\nfac 0 = 1\nfac n = n * fac (n - 1)");
    match first_func(&p) {
        Decl::Func { doc, name, .. } => {
            assert_eq!(name, "fac");
            assert_eq!(*doc, vec!["Factorial function.".to_string()]);
        }
        _ => unreachable!(),
    }
}

#[test]
fn doc_comment_multi_line() {
    let p = parse_ok("--- Sort a list.\n--- Uses quicksort.\nqsort x = x");
    match first_func(&p) {
        Decl::Func { doc, .. } => {
            assert_eq!(*doc, vec!["Sort a list.".to_string(), "Uses quicksort.".to_string()]);
        }
        _ => unreachable!(),
    }
}

#[test]
fn doc_comment_with_example() {
    let p = parse_ok("--- example: fac 5 == 120\nfac n = n");
    match first_func(&p) {
        Decl::Func { doc, .. } => {
            assert_eq!(*doc, vec!["example: fac 5 == 120".to_string()]);
        }
        _ => unreachable!(),
    }
}

#[test]
fn no_doc_comment() {
    let p = parse_ok("-- regular comment\nfac n = n");
    match first_func(&p) {
        Decl::Func { doc, .. } => {
            assert!(doc.is_empty());
        }
        _ => unreachable!(),
    }
}

#[test]
fn doc_comment_on_module() {
    let p = parse_ok("--- Vector module.\nmod Vec\n  make x = x");
    assert_eq!(p.modules[0].doc, vec!["Vector module.".to_string()]);
}

#[test]
fn doc_comment_on_adt() {
    let p = parse_ok("--- Optional value.\nMaybe a = Just a | None");
    let td = p.decls.iter().find(|d| matches!(d, Decl::TypeDef { .. })).unwrap();
    match td {
        Decl::TypeDef { doc, name, .. } => {
            assert_eq!(name, "Maybe");
            assert_eq!(*doc, vec!["Optional value.".to_string()]);
        }
        _ => unreachable!(),
    }
}

// ── Type Aliases ─────────────────────────────────────────

#[test]
fn type_alias_simple() {
    let p = parse_ok("type Pos = Int\nmain = 42");
    let alias = p.decls.iter().find(|d| matches!(d, Decl::TypeAlias { .. }));
    assert!(alias.is_some(), "should parse type alias");
    if let Some(Decl::TypeAlias { name, params, .. }) = alias {
        assert_eq!(name, "Pos");
        assert!(params.is_empty());
    }
}

#[test]
fn type_alias_parametric() {
    let p = parse_ok("type Pair a b = Int\nmain = 42");
    if let Some(Decl::TypeAlias { name, params, .. }) = p.decls.iter().find(|d| matches!(d, Decl::TypeAlias { .. })) {
        assert_eq!(name, "Pair");
        assert_eq!(params, &["a".to_string(), "b".to_string()]);
    } else {
        panic!("should parse parametric type alias");
    }
}

#[test]
fn type_alias_arrow_body() {
    let p = parse_ok("type Transform = Int -> Int\nmain = 42");
    if let Some(Decl::TypeAlias { name, body, .. }) = p.decls.iter().find(|d| matches!(d, Decl::TypeAlias { .. })) {
        assert_eq!(name, "Transform");
        assert!(matches!(body.kind, TypeExprKind::Arrow(_, _)));
    } else {
        panic!("should parse type alias with arrow body");
    }
}

// ── Error Recovery ───────────────────────────────────────

#[test]
fn error_recovery_multiple_decls() {
    // First decl has error, second should still parse
    let (program, errors) = crate::parse_recovering("foo x = x +\nbar y = y * 2").unwrap();
    assert!(!errors.is_empty(), "should have at least one error");
    // bar should still be parsed successfully
    let has_bar = program.decls.iter().any(|d| {
        matches!(d, Decl::Func { name, .. } if name == "bar")
    });
    assert!(has_bar, "bar should be parsed despite error in foo");
}

#[test]
fn error_recovery_collects_multiple_errors() {
    let (_, errors) = crate::parse_recovering("foo x = x +\nbaz = == ==").unwrap();
    assert!(errors.len() >= 1, "should collect errors from multiple broken decls, got {}", errors.len());
}

#[test]
fn error_recovery_valid_program_no_errors() {
    let (program, errors) = crate::parse_recovering("foo x = x + 1\nbar y = y * 2").unwrap();
    assert!(errors.is_empty(), "valid program should have no errors");
    assert_eq!(program.decls.len(), 2);
}

#[test]
fn error_recovery_partial_ast_usable() {
    // Error in middle, but first and last decls should parse
    let (program, errors) = crate::parse_recovering("good1 x = x\nbad = +\ngood2 y = y").unwrap();
    assert!(!errors.is_empty(), "should have error for bad decl");
    let names: Vec<_> = program.decls.iter().filter_map(|d| {
        if let Decl::Func { name, .. } = d { Some(name.as_str()) } else { None }
    }).collect();
    assert!(names.contains(&"good1"), "good1 should parse");
    assert!(names.contains(&"good2"), "good2 should parse");
}

// ── Import declarations ─────────────────────────────────

#[test]
fn import_decl() {
    let p = parse_ok("import \"math.sno\"\nmain = 42");
    assert_eq!(p.imports.len(), 1);
    assert_eq!(p.imports[0].path, "math.sno");
    assert_eq!(p.decls.len(), 1);
}

#[test]
fn import_multiple() {
    let p = parse_ok("import \"a.sno\"\nimport \"b.sno\"\nmain = 1");
    assert_eq!(p.imports.len(), 2);
    assert_eq!(p.imports[0].path, "a.sno");
    assert_eq!(p.imports[1].path, "b.sno");
}

#[test]
fn import_with_use() {
    let p = parse_ok("import \"math.sno\"\nuse Math (square)\nmain = square 5");
    assert_eq!(p.imports.len(), 1);
    assert_eq!(p.uses.len(), 1);
}

#[test]
fn no_imports_is_empty() {
    let p = parse_ok("main = 42");
    assert!(p.imports.is_empty());
}

// ── String Interpolation ─────────────────────────────────

#[test]
fn interp_desugars_to_show_concat() {
    // "hello ${name}" → "hello " ++ show name
    let e = expr_ok(r#""hello ${name}""#);
    match &e.kind {
        ExprKind::BinOp(BinOp::Concat, lhs, rhs) => {
            assert_eq!(lhs.kind, ExprKind::Lit(Lit::Str("hello ".into())));
            // rhs = App(show, name)
            match &rhs.kind {
                ExprKind::App(f, arg) => {
                    assert_eq!(f.kind, ExprKind::Var("show".into()));
                    assert_eq!(arg.kind, ExprKind::Var("name".into()));
                }
                other => panic!("Expected App, got {:?}", other),
            }
        }
        other => panic!("Expected BinOp(Concat), got {:?}", other),
    }
}

#[test]
fn interp_single_expr_becomes_show() {
    // "${x}" → show x
    let e = expr_ok(r#""${x}""#);
    match &e.kind {
        ExprKind::App(f, arg) => {
            assert_eq!(f.kind, ExprKind::Var("show".into()));
            assert_eq!(arg.kind, ExprKind::Var("x".into()));
        }
        other => panic!("Expected App(show, x), got {:?}", other),
    }
}

#[test]
fn interp_with_expression() {
    // "${x + 1}" → show (x + 1)
    let e = expr_ok(r#""${x + 1}""#);
    match &e.kind {
        ExprKind::App(f, _arg) => {
            assert_eq!(f.kind, ExprKind::Var("show".into()));
        }
        other => panic!("Expected App(show, ...), got {:?}", other),
    }
}

#[test]
fn interp_plain_string_unchanged() {
    // "hello" → Lit(Str("hello"))
    let e = expr_ok(r#""hello""#);
    assert_eq!(e.kind, ExprKind::Lit(Lit::Str("hello".into())));
}

#[test]
fn interp_escaped_dollar() {
    // "\$ money" → Lit(Str("$ money"))
    let e = expr_ok(r#""\$ money""#);
    assert_eq!(e.kind, ExprKind::Lit(Lit::Str("$ money".into())));
}

// ── Deriving ────────────────────────────────────────────

#[test]
fn derive_single_trait() {
    let p = parse_ok("Color = Red | Green | Blue derive (Show)\nmain = 1");
    let td = p.decls.iter().find(|d| matches!(d, Decl::TypeDef { .. })).unwrap();
    if let Decl::TypeDef { derives, .. } = td {
        assert_eq!(*derives, vec!["Show".to_string()]);
    } else { unreachable!(); }
}

#[test]
fn derive_multiple_traits() {
    let p = parse_ok("Color = Red | Green | Blue derive (Show, Eq, Ord)\nmain = 1");
    let td = p.decls.iter().find(|d| matches!(d, Decl::TypeDef { .. })).unwrap();
    if let Decl::TypeDef { derives, .. } = td {
        assert_eq!(*derives, vec!["Show".to_string(), "Eq".to_string(), "Ord".to_string()]);
    } else { unreachable!(); }
}

#[test]
fn no_derive_clause() {
    let p = parse_ok("Color = Red | Green | Blue\nmain = 1");
    let td = p.decls.iter().find(|d| matches!(d, Decl::TypeDef { .. })).unwrap();
    if let Decl::TypeDef { derives, .. } = td {
        assert!(derives.is_empty());
    } else { unreachable!(); }
}

#[test]
fn derive_unknown_trait_error() {
    let result = crate::parse("Color = Red | Green | Blue derive (Functor)\nmain = 1");
    assert!(result.is_err());
}

// ── Record Punning ──────────────────────────────────────────

#[test]
fn expr_record_punning() {
    let e = parse_expr("{x, y}").expect("parse failed");
    match e.kind {
        ExprKind::Record(fields) => {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "x");
            assert_eq!(fields[1].0, "y");
            assert!(matches!(&fields[0].1.kind, ExprKind::Var(n) if n == "x"));
            assert!(matches!(&fields[1].1.kind, ExprKind::Var(n) if n == "y"));
        }
        _ => panic!("Expected Record, got {:?}", e.kind),
    }
}

#[test]
fn expr_record_punning_mixed() {
    let e = parse_expr("{x, y, z = 42}").expect("parse failed");
    match e.kind {
        ExprKind::Record(fields) => {
            assert_eq!(fields.len(), 3);
            assert_eq!(fields[0].0, "x");
            assert!(matches!(&fields[0].1.kind, ExprKind::Var(n) if n == "x"));
            assert_eq!(fields[2].0, "z");
            assert!(matches!(&fields[2].1.kind, ExprKind::Lit(Lit::Int(42))));
        }
        _ => panic!("Expected Record"),
    }
}

#[test]
fn expr_record_punning_single() {
    let e = parse_expr("{x}").expect("parse failed");
    match e.kind {
        ExprKind::Record(fields) => {
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].0, "x");
            assert!(matches!(&fields[0].1.kind, ExprKind::Var(n) if n == "x"));
        }
        _ => panic!("Expected Record"),
    }
}

#[test]
fn pattern_record_punning() {
    let p = parse_ok("f {x, y} = x + y");
    match first_func(&p) {
        Decl::Func { equations, .. } => {
            if let Pat::Record(fields) = &equations[0].pats[0] {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "x");
                assert!(matches!(&fields[0].1, Pat::Var(n) if n == "x"));
                assert_eq!(fields[1].0, "y");
                assert!(matches!(&fields[1].1, Pat::Var(n) if n == "y"));
            } else {
                panic!("Expected record pattern");
            }
        }
        _ => unreachable!(),
    }
}

#[test]
fn pattern_record_punning_mixed() {
    let p = parse_ok("f {x, y = _} = x");
    match first_func(&p) {
        Decl::Func { equations, .. } => {
            if let Pat::Record(fields) = &equations[0].pats[0] {
                assert_eq!(fields.len(), 2);
                assert!(matches!(&fields[0].1, Pat::Var(n) if n == "x"));
                assert!(matches!(&fields[1].1, Pat::Wildcard));
            } else {
                panic!("Expected record pattern");
            }
        }
        _ => unreachable!(),
    }
}

// ── Wildcard Import ─────────────────────────────────────────

#[test]
fn parse_use_wildcard() {
    let src = "mod Math\n  square x = x * x\n\nuse Math (*)\n\nmain = square 5";
    let p = parse_ok(src);
    assert_eq!(p.uses.len(), 1);
    assert_eq!(p.uses[0].module, "Math");
    assert_eq!(p.uses[0].names, vec!["*"]);
}

#[test]
fn parse_use_selective_unchanged() {
    let src = "mod Math\n  square x = x * x\n\nuse Math (square)\n\nmain = square 5";
    let p = parse_ok(src);
    assert_eq!(p.uses[0].names, vec!["square"]);
}

// ── Test declarations ─────────────────────────────────────

#[test]
fn parse_test_decl() {
    let src = "f x = x + 1\ntest \"inc works\" = f 0 == 1";
    let p = parse_ok(src);
    let test = p.decls.iter().find(|d| matches!(d, Decl::Test { .. }));
    assert!(test.is_some());
    if let Decl::Test { name, .. } = test.unwrap() {
        assert_eq!(name, "inc works");
    }
}

#[test]
fn parse_test_with_prop() {
    let src = "f x = x\ntest \"identity\" = prop x -> f x == x";
    let p = parse_ok(src);
    let test = p.decls.iter().find(|d| matches!(d, Decl::Test { .. }));
    assert!(test.is_some());
    if let Decl::Test { body, .. } = test.unwrap() {
        assert!(matches!(body.kind, ExprKind::Prop(_, _)));
    }
}

#[test]
fn parse_test_with_when() {
    let e = expr_ok("x + 1 > 1 when x > 0");
    assert!(matches!(e.kind, ExprKind::When(_, _)));
}

#[test]
fn parse_when_precedence() {
    // when binds looser than ==
    let e = expr_ok("a == b when c == d");
    if let ExprKind::When(lhs, rhs) = &e.kind {
        assert!(matches!(lhs.kind, ExprKind::BinOp(BinOp::Eq, _, _)));
        assert!(matches!(rhs.kind, ExprKind::BinOp(BinOp::Eq, _, _)));
    } else {
        panic!("Expected Implies");
    }
}

#[test]
fn parse_prop_multi_var() {
    let e = expr_ok("prop x y -> x + y == y + x");
    if let ExprKind::Prop(vars, _) = &e.kind {
        assert_eq!(vars, &["x".to_string(), "y".to_string()]);
    } else {
        panic!("Expected Prop");
    }
}

