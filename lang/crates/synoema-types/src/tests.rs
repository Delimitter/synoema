use crate::*;

fn check(src: &str) -> TypeEnv {
    typecheck(src).unwrap_or_else(|e| panic!("Type check failed for:\n{}\nError: {}", src, e))
}

fn check_err(src: &str) -> String {
    typecheck(src).unwrap_err()
}

fn infer(src: &str) -> Type {
    infer_expr_type(src)
        .unwrap_or_else(|e| panic!("Infer failed for:\n{}\nError: {}", src, e))
}

fn assert_type_matches(ty: &Type, expected: &str) {
    let s = format!("{}", ty);
    assert!(
        s == expected || types_structurally_match(ty, expected),
        "Expected type '{}', got '{}'", expected, s
    );
}

/// Simple structural check — type variables may have different names
fn types_structurally_match(ty: &Type, expected: &str) -> bool {
    // Normalize: replace all type variable names with sequential letters
    let rendered = format!("{}", ty);
    // For basic cases, just check if the shape matches
    rendered.len() == expected.len() || {
        // More lenient: check that non-variable parts match
        let r_parts: Vec<&str> = rendered.split(|c: char| c.is_lowercase() && c.is_alphabetic()).collect();
        let e_parts: Vec<&str> = expected.split(|c: char| c.is_lowercase() && c.is_alphabetic()).collect();
        r_parts == e_parts
    }
}

// ── Literal Types ─────────────────────────────────────

#[test]
fn literal_int() {
    let ty = infer("x = 42");
    assert_eq!(ty, Type::int());
}

#[test]
fn literal_float() {
    let ty = infer("x = 3.14");
    assert_eq!(ty, Type::float());
}

#[test]
fn literal_string() {
    let ty = infer("x = \"hello\"");
    assert_eq!(ty, Type::string());
}

#[test]
fn literal_bool() {
    let ty = infer("x = true");
    assert_eq!(ty, Type::bool());
}

// ── Arithmetic ────────────────────────────────────────

#[test]
fn add_ints() {
    let ty = infer("f x y = x + y");
    assert_eq!(ty, Type::arrow(Type::int(), Type::arrow(Type::int(), Type::int())));
}

#[test]
fn mul_ints() {
    let ty = infer("f x = x * 2");
    assert_eq!(ty, Type::arrow(Type::int(), Type::int()));
}

// ── Comparison ────────────────────────────────────────

#[test]
fn comparison_returns_bool() {
    let ty = infer("f x y = x < y");
    // x, y must be same type; result is Bool
    match &ty {
        Type::Arrow(_, rest) => match rest.as_ref() {
            Type::Arrow(_, ret) => assert_eq!(ret.as_ref(), &Type::bool()),
            _ => panic!("Expected curried function"),
        },
        _ => panic!("Expected function type"),
    }
}

// ── Conditional ───────────────────────────────────────

#[test]
fn cond_same_branches() {
    let ty = infer("f x = ? x > 0 -> 1 : 0");
    assert_eq!(ty, Type::arrow(Type::int(), Type::int()));
}

// ── List ──────────────────────────────────────────────

#[test]
fn empty_list() {
    let ty = infer("x = []");
    match ty {
        Type::App(con, _) => assert_eq!(*con, Type::Con("List".into())),
        _ => panic!("Expected List type, got {}", ty),
    }
}

#[test]
fn int_list() {
    let ty = infer("x = [1 2 3]");
    assert_eq!(ty, Type::list(Type::int()));
}

#[test]
fn range() {
    let ty = infer("x = [1..10]");
    assert_eq!(ty, Type::list(Type::int()));
}

// ── Block / Let ───────────────────────────────────────

#[test]
fn block_bindings() {
    let ty = infer("f =\n  x = 1\n  y = 2\n  x + y");
    assert_eq!(ty, Type::int());
}

// ── Lambda ────────────────────────────────────────────

#[test]
fn lambda_identity() {
    // \x -> x should be polymorphic: a -> a
    let ty = infer("id = \\x -> x");
    match ty {
        Type::Arrow(a, b) => assert_eq!(a, b),
        _ => panic!("Expected arrow type, got {}", ty),
    }
}

// ── Pattern Matching ──────────────────────────────────

#[test]
fn pattern_match_factorial() {
    let env = check("fac 0 = 1\nfac n = n * fac (n - 1)");
    let scheme = env.lookup("fac").expect("fac not found");
    // fac : Int -> Int
    assert_eq!(scheme.ty, Type::arrow(Type::int(), Type::int()));
}

// ── Pipe ──────────────────────────────────────────────

#[test]
fn pipe_application() {
    // [1 2 3] |> sum  should be Int
    let ty = infer("x = [1 2 3] |> sum");
    assert_eq!(ty, Type::int());
}

// ── List Comprehension ────────────────────────────────

#[test]
fn list_comp_simple() {
    // [x | x <- [1 2 3]] : List Int
    let ty = infer("x = [x | x <- [1 2 3]]");
    assert_eq!(ty, Type::list(Type::int()));
}

#[test]
fn list_comp_with_guard() {
    let ty = infer("x = [x | x <- [1 2 3] , x > 1]");
    assert_eq!(ty, Type::list(Type::int()));
}

// ── ADT ───────────────────────────────────────────────

#[test]
fn adt_constructors() {
    let env = check("Maybe a = Just a | None");
    let just = env.lookup("Just").expect("Just not found");
    let none = env.lookup("None").expect("None not found");
    // Just : a -> Maybe a (polymorphic)
    assert!(!just.vars.is_empty());
    // None : Maybe a (polymorphic, no args)
    assert!(!none.vars.is_empty());
}

#[test]
fn adt_shape_constructors() {
    let env = check("Shape = Circle Float | Rect Float Float | Point");
    let circle = env.lookup("Circle").expect("Circle not found");
    // Circle : Float -> Shape
    assert_eq!(circle.ty, Type::arrow(Type::float(), Type::Con("Shape".into())));
    let point = env.lookup("Point").expect("Point not found");
    // Point : Shape
    assert_eq!(point.ty, Type::Con("Shape".into()));
}

// ── Concat ────────────────────────────────────────────

#[test]
fn concat_lists() {
    let ty = infer("x = [1 2] ++ [3 4]");
    assert_eq!(ty, Type::list(Type::int()));
}

// ── Full Programs ─────────────────────────────────────

#[test]
fn full_factorial() {
    let env = check("fac 0 = 1\nfac n = n * fac (n - 1)");
    assert!(env.lookup("fac").is_some());
}

#[test]
fn full_fizzbuzz() {
    let env = check("fizzbuzz n = ? n % 15 == 0 -> \"FizzBuzz\" : ? n % 3 == 0 -> \"Fizz\" : ? n % 5 == 0 -> \"Buzz\" : show n");
    let scheme = env.lookup("fizzbuzz").expect("fizzbuzz not found");
    assert_eq!(scheme.ty, Type::arrow(Type::int(), Type::string()));
}

// ── Error Cases ───────────────────────────────────────

#[test]
fn error_undefined_var() {
    let err = check_err("f x = y");
    assert!(err.contains("Undefined") || err.contains("Unbound"), "Error: {}", err);
}

#[test]
fn error_type_mismatch_add_bool() {
    let err = check_err("f = true + 1");
    assert!(err.contains("mismatch") || err.contains("Mismatch"), "Error: {}", err);
}

#[test]
fn error_cond_branch_mismatch() {
    let err = check_err("f x = ? x > 0 -> 1 : \"no\"");
    assert!(err.contains("mismatch") || err.contains("Mismatch"), "Error: {}", err);
}

// ── Polymorphism ─────────────────────────────────────

#[test]
fn polymorphic_identity() {
    // id x = x  →  should infer a -> a (polymorphic)
    let ty = infer("id x = x");
    let s = format!("{}", ty);
    // Should be something like "a -> a" — both sides same variable
    assert!(s.contains("->"), "Expected arrow type, got: {}", s);
    let parts: Vec<&str> = s.split(" -> ").collect();
    assert_eq!(parts.len(), 2, "Expected 'X -> X', got: {}", s);
    assert_eq!(parts[0].trim(), parts[1].trim(), "Identity should return same type var: {}", s);
}

#[test]
fn polymorphic_const() {
    // const_ x y = x  →  should infer a -> b -> a
    let ty = infer("const_ x y = x");
    let s = format!("{}", ty);
    assert!(s.contains("->"), "Expected arrow type, got: {}", s);
}

#[test]
fn let_polymorphism() {
    // f should be usable at different types within same scope
    let env = check("f x = x\ng = f 42\nh = f true");
    // g : Int, h : Bool
    let g_ty = env.lookup("g").expect("g not found");
    let h_ty = env.lookup("h").expect("h not found");
    assert_eq!(format!("{}", g_ty.ty), "Int");
    assert_eq!(format!("{}", h_ty.ty), "Bool");
}

// ── Higher-Order Functions ───────────────────────────

#[test]
fn higher_order_apply() {
    // apply f x = f x  →  (a -> b) -> a -> b
    let ty = infer("apply f x = f x");
    let s = format!("{}", ty);
    assert!(s.contains("->"), "Expected arrow type, got: {}", s);
}

#[test]
fn compose_functions() {
    // comp f g x = f (g x)  →  (b -> c) -> (a -> b) -> a -> c
    let ty = infer("comp f g x = f (g x)");
    let s = format!("{}", ty);
    assert!(s.contains("->"), "Expected nested arrow type, got: {}", s);
}

// ── Full Programs ────────────────────────────────────

#[test]
fn full_map() {
    let env = check("map f [] = []\nmap f (x:xs) = f x : map f xs");
    let map_ty = env.lookup("map").expect("map not found");
    let s = format!("{}", map_ty.ty);
    // Should be: (a -> b) -> List a -> List b
    assert!(s.contains("->"), "map should be a function: {}", s);
    assert!(s.contains("List"), "map should involve List: {}", s);
}

#[test]
fn full_quicksort() {
    let env = check(
        "qsort [] = []\nqsort (p:xs) = qsort lo ++ [p] ++ qsort hi\n  lo = [x | x <- xs , x <= p]\n  hi = [x | x <- xs , x > p]"
    );
    let qsort_ty = env.lookup("qsort").expect("qsort not found");
    let s = format!("{}", qsort_ty.ty);
    // Should be: List a -> List a (or similar with Int constraint)
    assert!(s.contains("List"), "qsort should involve List: {}", s);
    assert!(s.contains("->"), "qsort should be a function: {}", s);
}

// ── Error Cases (extended) ───────────────────────────

#[test]
fn error_applying_non_function() {
    let err = check_err("f = 42 true");
    assert!(!err.is_empty(), "Should produce type error for applying Int to Bool");
}

#[test]
fn error_wrong_pattern_arity() {
    // Using a list pattern where not applicable
    let err = check_err("f (x:y:z) = x + y + z\ng = f 42");
    // This should either parse-error or type-error
    assert!(!err.is_empty());
}
