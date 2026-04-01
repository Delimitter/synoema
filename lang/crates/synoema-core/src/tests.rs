use crate::*;

fn core(src: &str) -> CoreProgram {
    to_core(src).unwrap_or_else(|e| panic!("Failed to desugar:\n{}\nError: {}", src, e))
}

fn core_def(src: &str) -> CoreExpr {
    let prog = core(src);
    prog.defs.last().expect("No defs in program").body.clone()
}

fn fmt(src: &str) -> String {
    format!("{}", core_def(src))
}

// ── Literals & Variables ────────────────────────────────

#[test]
fn desugar_int_constant() {
    let body = core_def("x = 42");
    assert_eq!(body, CoreExpr::Lit(synoema_parser::Lit::Int(42)));
}

#[test]
fn desugar_string_constant() {
    let body = core_def("x = \"hello\"");
    assert_eq!(body, CoreExpr::Lit(synoema_parser::Lit::Str("hello".into())));
}

#[test]
fn desugar_bool_constant() {
    let body = core_def("x = true");
    assert_eq!(body, CoreExpr::Lit(synoema_parser::Lit::Bool(true)));
}

// ── Simple Functions → Lambda ───────────────────────────

#[test]
fn desugar_identity() {
    let s = fmt("id x = x");
    // Should be: (\x -> x)
    assert!(s.contains("\\"), "Should contain lambda: {}", s);
    assert!(s.contains("x"), "Should contain x: {}", s);
}

#[test]
fn desugar_two_args() {
    let s = fmt("add x y = x");
    // Should be nested lambdas: (\x -> (\y -> x))
    let lambda_count = s.matches('\\').count();
    assert_eq!(lambda_count, 2, "Expected 2 lambdas, got: {}", s);
}

// ── BinOps → PrimOp Application ────────────────────────

#[test]
fn desugar_addition() {
    let s = fmt("f x = x + 1");
    assert!(s.contains("add#"), "Should contain add# primop: {}", s);
}

#[test]
fn desugar_multiply() {
    let s = fmt("f x = x * 2");
    assert!(s.contains("mul#"), "Should contain mul# primop: {}", s);
}

#[test]
fn desugar_comparison() {
    let s = fmt("f x = x > 0");
    assert!(s.contains("gt#"), "Should contain gt# primop: {}", s);
}

// ── Negation → PrimOp ──────────────────────────────────

#[test]
fn desugar_neg() {
    let s = fmt("f x = -x");
    assert!(s.contains("neg#"), "Should contain neg# primop: {}", s);
}

// ── Pipe → Application ─────────────────────────────────

#[test]
fn desugar_pipe() {
    // x |> f  →  App(f, x)
    let body = core_def("r = x |> f");
    match &body {
        CoreExpr::App(func, arg) => {
            assert!(matches!(func.as_ref(), CoreExpr::Var(n) if n == "f"));
            assert!(matches!(arg.as_ref(), CoreExpr::Var(n) if n == "x"));
        }
        _ => panic!("Expected App, got: {:?}", body),
    }
}

// ── Compose → Lambda ────────────────────────────────────

#[test]
fn desugar_compose() {
    let s = fmt("r = f >> g");
    // f >> g  →  \x -> g (f x)
    assert!(s.contains("\\"), "Should contain lambda: {}", s);
}

// ── Conditional → Case Bool ─────────────────────────────

#[test]
fn desugar_cond() {
    let body = core_def("f x = ? x > 0 -> x : 0");
    match &body {
        CoreExpr::Lam(_, inner) => match inner.as_ref() {
            CoreExpr::Case(_, alts) => {
                assert_eq!(alts.len(), 2, "Cond should desugar to 2 alts");
                assert_eq!(alts[0].pat, CorePat::Lit(synoema_parser::Lit::Bool(true)));
                assert_eq!(alts[1].pat, CorePat::Lit(synoema_parser::Lit::Bool(false)));
            }
            _ => panic!("Expected Case inside Lam, got: {:?}", inner),
        },
        _ => panic!("Expected Lam, got: {:?}", body),
    }
}

// ── Block → Nested Let ──────────────────────────────────

#[test]
fn desugar_block() {
    let body = core_def("f =\n  a = 10\n  b = 20\n  a");
    // Should be: Let(a, 10, Let(b, 20, a))
    match &body {
        CoreExpr::Let(name1, val1, inner) => {
            assert_eq!(name1, "a");
            assert_eq!(**val1, CoreExpr::Lit(synoema_parser::Lit::Int(10)));
            match inner.as_ref() {
                CoreExpr::Let(name2, val2, result) => {
                    assert_eq!(name2, "b");
                    assert_eq!(**val2, CoreExpr::Lit(synoema_parser::Lit::Int(20)));
                    assert_eq!(**result, CoreExpr::Var("a".into()));
                }
                _ => panic!("Expected inner Let, got: {:?}", inner),
            }
        }
        _ => panic!("Expected Let, got: {:?}", body),
    }
}

// ── List → MkList ───────────────────────────────────────

#[test]
fn desugar_list() {
    let body = core_def("xs = [1 2 3]");
    match &body {
        CoreExpr::MkList(elems) => {
            assert_eq!(elems.len(), 3);
        }
        _ => panic!("Expected MkList, got: {:?}", body),
    }
}

#[test]
fn desugar_empty_list() {
    let body = core_def("xs = []");
    match &body {
        CoreExpr::MkList(elems) => assert_eq!(elems.len(), 0),
        _ => panic!("Expected MkList, got: {:?}", body),
    }
}

// ── Range → PrimOp ──────────────────────────────────────

#[test]
fn desugar_range() {
    let s = fmt("xs = [1..10]");
    assert!(s.contains("range#"), "Should contain range# primop: {}", s);
}

// ── Pattern Match → Case ────────────────────────────────

#[test]
fn desugar_pattern_match_single_arg() {
    let body = core_def("fac 0 = 1\nfac n = n");
    // Should be: Lam(arg, Case(arg, [0 -> 1, n -> n]))
    match &body {
        CoreExpr::Lam(_, inner) => match inner.as_ref() {
            CoreExpr::Case(_, alts) => {
                assert_eq!(alts.len(), 2, "Expected 2 alternatives");
                assert_eq!(alts[0].pat, CorePat::Lit(synoema_parser::Lit::Int(0)));
            }
            _ => panic!("Expected Case, got: {:?}", inner),
        },
        _ => panic!("Expected Lam, got: {:?}", body),
    }
}

#[test]
fn desugar_cons_pattern() {
    let body = core_def("head (x:xs) = x");
    let s = format!("{}", body);
    // Should contain Cons pattern
    assert!(s.contains("Cons"), "Should desugar cons pattern: {}", s);
}

// ── List Comprehension → concatMap ──────────────────────

#[test]
fn desugar_list_comp() {
    let s = fmt("xs = [x | x <- ys]");
    assert!(s.contains("concatMap"), "Should contain concatMap: {}", s);
}

#[test]
fn desugar_list_comp_with_guard() {
    let s = fmt("xs = [x | x <- ys , x > 0]");
    assert!(s.contains("concatMap"), "Should contain concatMap: {}", s);
    // Guard desugars to case on Bool
    assert!(s.contains("true") || s.contains("case"), "Should contain guard case: {}", s);
}

// ── Full Programs ───────────────────────────────────────

#[test]
fn full_factorial() {
    let prog = core("fac 0 = 1\nfac n = n * fac (n - 1)");
    assert_eq!(prog.defs.len(), 1);
    assert_eq!(prog.defs[0].name, "fac");
    let s = format!("{}", prog.defs[0].body);
    assert!(s.contains("case"), "Factorial should have case: {}", s);
    assert!(s.contains("mul#"), "Factorial should have mul#: {}", s);
}

#[test]
fn full_map() {
    let prog = core("map f [] = []\nmap f (x:xs) = f x : map f xs");
    assert_eq!(prog.defs.len(), 1);
    assert_eq!(prog.defs[0].name, "map");
}

#[test]
fn full_quicksort() {
    let prog = core(
        "qsort [] = []\nqsort (p:xs) = qsort lo ++ [p] ++ qsort hi\n  lo = [x | x <- xs , x <= p]\n  hi = [x | x <- xs , x > p]"
    );
    assert_eq!(prog.defs.len(), 1);
    assert_eq!(prog.defs[0].name, "qsort");
    let s = format!("{}", prog.defs[0].body);
    assert!(s.contains("concat#"), "Quicksort should have concat#: {}", s);
    assert!(s.contains("concatMap"), "Quicksort should have concatMap: {}", s);
}

#[test]
fn full_fizzbuzz() {
    let prog = core(
        "fizzbuzz n =\n  ? n % 15 == 0 -> \"FizzBuzz\"\n  : ? n % 3 == 0 -> \"Fizz\"\n  : ? n % 5 == 0 -> \"Buzz\"\n  : show n"
    );
    assert_eq!(prog.defs.len(), 1);
    let s = format!("{}", prog.defs[0].body);
    assert!(s.contains("case"), "FizzBuzz should have case: {}", s);
    assert!(s.contains("mod#"), "FizzBuzz should have mod#: {}", s);
}

#[test]
fn full_compose() {
    let prog = core(
        "compose f g x = f (g x)\ndouble x = x * 2\ninc x = x + 1\nmain = compose double inc 5"
    );
    assert_eq!(prog.defs.len(), 4);
    let names: Vec<&str> = prog.defs.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"compose"));
    assert!(names.contains(&"double"));
    assert!(names.contains(&"inc"));
    assert!(names.contains(&"main"));
}

// ── Display / Pretty Print ──────────────────────────────

#[test]
fn display_round_trip() {
    // Just verify Display doesn't panic on various constructs
    let programs = vec![
        "x = 42",
        "f x = x + 1",
        "g = [1 2 3]",
        "h x = ? x > 0 -> x : 0",
        "fac 0 = 1\nfac n = n * fac (n - 1)",
    ];
    for src in programs {
        let prog = core(src);
        for def in &prog.defs {
            let s = format!("{}: {}", def.name, def.body);
            assert!(!s.is_empty(), "Display should produce output for: {}", src);
        }
    }
}
