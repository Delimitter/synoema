use crate::*;

fn ev(src: &str) -> Value {
    eval_expr(src).unwrap_or_else(|e| panic!("Eval failed for: {}\nError: {}", src, e))
}

fn run_main(src: &str) -> (Value, Vec<String>) {
    eval_main(src).unwrap_or_else(|e| panic!("Run failed for:\n{}\nError: {}", src, e))
}

fn run_ok(src: &str) -> Env {
    run(src).unwrap_or_else(|e| panic!("Run failed for:\n{}\nError: {}", src, e))
}

#[allow(dead_code)]
fn lookup(src: &str, name: &str) -> Value {
    let env = run_ok(src);
    env.lookup(name).cloned()
        .unwrap_or_else(|| panic!("'{}' not found in env", name))
}

// ── Literals ──────────────────────────────────────────

#[test]
fn lit_int() { assert_eq!(ev("42"), Value::Int(42)); }

#[test]
fn lit_float() { assert_eq!(ev("3.14"), Value::Float(3.14)); }

#[test]
fn lit_bool() {
    assert_eq!(ev("true"), Value::Bool(true));
    assert_eq!(ev("false"), Value::Bool(false));
}

#[test]
fn lit_string() { assert_eq!(ev("\"hello\""), Value::Str("hello".into())); }

// ── Arithmetic ────────────────────────────────────────

#[test]
fn add() { assert_eq!(ev("2 + 3"), Value::Int(5)); }

#[test]
fn sub() { assert_eq!(ev("10 - 4"), Value::Int(6)); }

#[test]
fn mul() { assert_eq!(ev("6 * 7"), Value::Int(42)); }

#[test]
fn div() { assert_eq!(ev("15 / 3"), Value::Int(5)); }

#[test]
fn modulo() { assert_eq!(ev("17 % 5"), Value::Int(2)); }

#[test]
fn neg() { assert_eq!(ev("-42"), Value::Int(-42)); }

#[test]
fn precedence() { assert_eq!(ev("2 + 3 * 4"), Value::Int(14)); }

#[test]
fn parens() { assert_eq!(ev("(2 + 3) * 4"), Value::Int(20)); }

// ── Comparison ────────────────────────────────────────

#[test]
fn eq() { assert_eq!(ev("1 == 1"), Value::Bool(true)); }

#[test]
fn neq() { assert_eq!(ev("1 != 2"), Value::Bool(true)); }

#[test]
fn lt() { assert_eq!(ev("1 < 2"), Value::Bool(true)); }

#[test]
fn gt() { assert_eq!(ev("2 > 1"), Value::Bool(true)); }

#[test]
fn lte() { assert_eq!(ev("1 <= 1"), Value::Bool(true)); }

#[test]
fn gte() { assert_eq!(ev("2 >= 3"), Value::Bool(false)); }

// ── Logic ─────────────────────────────────────────────

#[test]
fn and_true() { assert_eq!(ev("true && true"), Value::Bool(true)); }

#[test]
fn and_false() { assert_eq!(ev("true && false"), Value::Bool(false)); }

#[test]
fn or_true() { assert_eq!(ev("false || true"), Value::Bool(true)); }

// ── Lists ─────────────────────────────────────────────

#[test]
fn list_literal() {
    assert_eq!(ev("[1 2 3]"), Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
}

#[test]
fn empty_list() {
    assert_eq!(ev("[]"), Value::List(vec![]));
}

#[test]
fn list_concat() {
    assert_eq!(
        ev("[1 2] ++ [3 4]"),
        Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)])
    );
}

#[test]
fn range() {
    assert_eq!(
        ev("[1..5]"),
        Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4), Value::Int(5)])
    );
}

// ── Conditional ───────────────────────────────────────

#[test]
fn cond_true() { assert_eq!(ev("? true -> 1 : 2"), Value::Int(1)); }

#[test]
fn cond_false() { assert_eq!(ev("? false -> 1 : 2"), Value::Int(2)); }

#[test]
fn cond_comparison() { assert_eq!(ev("? 5 > 3 -> 10 : 20"), Value::Int(10)); }

// ── Lambda ────────────────────────────────────────────

#[test]
fn lambda_apply() {
    assert_eq!(ev("(\\x -> x + 1) 5"), Value::Int(6));
}

#[test]
fn lambda_closure() {
    // Lambda captures enclosing scope
    let (_v, _) = run_main("a = 10\nf = \\x -> x + a\nresult = f 5");
    // result = 15
    // But eval_main returns the last func which is 'result'
    // We need to look it up properly
    let env = run_ok("a = 10\nf = \\x -> x + a\nresult = f 5");
    match env.lookup("result") {
        Some(Value::Func { equations, .. }) => {
            let mut evaluator = Evaluator::new();
            let v = evaluator.eval(&env, &equations[0].body).unwrap();
            assert_eq!(v, Value::Int(15));
        }
        other => panic!("Expected result func, got: {:?}", other),
    }
}

// ── Pipe ──────────────────────────────────────────────

#[test]
fn pipe_simple() {
    let env = run_ok("double x = x * 2\nresult = 5 |> double");
    match env.lookup("result") {
        Some(Value::Func { equations, .. }) => {
            let mut evaluator = Evaluator::new();
            let v = evaluator.eval(&env, &equations[0].body).unwrap();
            assert_eq!(v, Value::Int(10));
        }
        other => panic!("Expected result, got: {:?}", other),
    }
}

// ── Pattern Matching ──────────────────────────────────

#[test]
fn pattern_match_literal() {
    let env = run_ok("f 0 = 100\nf n = n");
    let mut ev = Evaluator::new();
    let f = env.lookup("f").unwrap().clone();
    assert_eq!(ev.apply(f.clone(), Value::Int(0)).unwrap(), Value::Int(100));
    assert_eq!(ev.apply(f, Value::Int(5)).unwrap(), Value::Int(5));
}

#[test]
fn pattern_match_list() {
    let env = run_ok("len [] = 0\nlen (x:xs) = 1 + len xs");
    let mut ev = Evaluator::new();
    let f = env.lookup("len").unwrap().clone();
    let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    assert_eq!(ev.apply(f, list).unwrap(), Value::Int(3));
}

// ── Full Programs ─────────────────────────────────────

#[test]
fn full_factorial() {
    let env = run_ok("fac 0 = 1\nfac n = n * fac (n - 1)");
    let mut ev = Evaluator::new();
    let f = env.lookup("fac").unwrap().clone();
    assert_eq!(ev.apply(f.clone(), Value::Int(0)).unwrap(), Value::Int(1));
    assert_eq!(ev.apply(f.clone(), Value::Int(1)).unwrap(), Value::Int(1));
    assert_eq!(ev.apply(f.clone(), Value::Int(5)).unwrap(), Value::Int(120));
    assert_eq!(ev.apply(f, Value::Int(10)).unwrap(), Value::Int(3628800));
}

#[test]
fn full_map() {
    let env = run_ok("map f [] = []\nmap f (x:xs) = f x : map f xs\ndouble x = x * 2");
    let mut ev = Evaluator::new();
    let map_f = env.lookup("map").unwrap().clone();
    let double = env.lookup("double").unwrap().clone();
    let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);

    let partial = ev.apply(map_f, double).unwrap();
    let result = ev.apply(partial, list).unwrap();
    assert_eq!(result, Value::List(vec![Value::Int(2), Value::Int(4), Value::Int(6)]));
}

#[test]
fn full_quicksort() {
    let src = "\
qsort [] = []
qsort (p:xs) = qsort lo ++ [p] ++ qsort hi
  lo = [x | x <- xs , x <= p]
  hi = [x | x <- xs , x > p]";
    let env = run_ok(src);
    let mut ev = Evaluator::new();
    let f = env.lookup("qsort").unwrap().clone();

    let unsorted = Value::List(vec![
        Value::Int(3), Value::Int(1), Value::Int(4), Value::Int(1), Value::Int(5), Value::Int(9),
    ]);
    let sorted = ev.apply(f, unsorted).unwrap();
    assert_eq!(sorted, Value::List(vec![
        Value::Int(1), Value::Int(1), Value::Int(3), Value::Int(4), Value::Int(5), Value::Int(9),
    ]));
}

#[test]
fn full_fizzbuzz_single() {
    let src = "\
fizzbuzz n =
  ? n % 15 == 0 -> \"FizzBuzz\"
  : ? n % 3 == 0 -> \"Fizz\"
  : ? n % 5 == 0 -> \"Buzz\"
  : show n";
    let env = run_ok(src);
    let mut ev = Evaluator::new();
    let f = env.lookup("fizzbuzz").unwrap().clone();

    assert_eq!(ev.apply(f.clone(), Value::Int(15)).unwrap(), Value::Str("FizzBuzz".into()));
    assert_eq!(ev.apply(f.clone(), Value::Int(3)).unwrap(), Value::Str("Fizz".into()));
    assert_eq!(ev.apply(f.clone(), Value::Int(5)).unwrap(), Value::Str("Buzz".into()));
    assert_eq!(ev.apply(f, Value::Int(7)).unwrap(), Value::Str("7".into()));
}

// ── List Comprehension ────────────────────────────────

#[test]
fn list_comp_simple() {
    let src = "squares = [x | x <- [1..5]]";
    // Note: this just extracts elements, no transformation in body
    let env = run_ok(src);
    match env.lookup("squares") {
        Some(Value::Func { equations, .. }) => {
            let mut ev = Evaluator::new();
            let v = ev.eval(&env, &equations[0].body).unwrap();
            assert_eq!(v, Value::List(vec![
                Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4), Value::Int(5)
            ]));
        }
        _ => panic!("squares not found"),
    }
}

#[test]
fn list_comp_with_guard() {
    let src = "evens = [x | x <- [1..10] , even x]";
    let env = run_ok(src);
    match env.lookup("evens") {
        Some(Value::Func { equations, .. }) => {
            let mut ev = Evaluator::new();
            let v = ev.eval(&env, &equations[0].body).unwrap();
            assert_eq!(v, Value::List(vec![
                Value::Int(2), Value::Int(4), Value::Int(6), Value::Int(8), Value::Int(10)
            ]));
        }
        _ => panic!("evens not found"),
    }
}

// ── ADT ───────────────────────────────────────────────

#[test]
fn adt_constructors() {
    let src = "Maybe a = Just a | None\nx = Just 42\ny = None";
    let env = run_ok(src);
    let mut ev = Evaluator::new();

    // x = Just 42
    match env.lookup("x") {
        Some(Value::Func { equations, .. }) => {
            let v = ev.eval(&env, &equations[0].body).unwrap();
            assert_eq!(v, Value::Con("Just".into(), vec![Value::Int(42)]));
        }
        _ => panic!("x not found"),
    }
}

// ── Builtins ──────────────────────────────────────────

#[test]
fn builtin_length() {
    assert_eq!(ev("length [1 2 3]"), Value::Int(3));
}

#[test]
fn builtin_head_tail() {
    assert_eq!(ev("head [1 2 3]"), Value::Int(1));
    assert_eq!(
        ev("tail [1 2 3]"),
        Value::List(vec![Value::Int(2), Value::Int(3)])
    );
}

#[test]
fn builtin_sum() {
    assert_eq!(ev("sum [1 2 3 4 5]"), Value::Int(15));
}

#[test]
fn builtin_even_odd() {
    assert_eq!(ev("even 4"), Value::Bool(true));
    assert_eq!(ev("even 3"), Value::Bool(false));
    assert_eq!(ev("odd 3"), Value::Bool(true));
}

// ── Error Cases ───────────────────────────────────────

#[test]
fn error_div_zero() {
    assert!(eval_expr("1 / 0").is_err());
}

#[test]
fn error_head_empty() {
    assert!(eval_expr("head []").is_err());
}

// ── Records (Phase 9.4) ───────────────────────────────

#[test]
fn record_literal() {
    let val = ev("{x = 3, y = 4}");
    assert_eq!(val, Value::Record(vec![
        ("x".into(), Value::Int(3)),
        ("y".into(), Value::Int(4)),
    ]));
}

#[test]
fn record_field_access() {
    let env = run_ok("p = {x = 10, y = 20}\nmain = p.x");
    match env.lookup("main") {
        Some(Value::Func { equations, .. }) => {
            let mut ev = Evaluator::new();
            let v = ev.eval(&env, &equations[0].body).unwrap();
            assert_eq!(v, Value::Int(10));
        }
        _ => panic!("main not found"),
    }
}

#[test]
fn record_pattern_match() {
    let env = run_ok("get_x {x = v, y = _} = v\nmain = get_x {x = 42, y = 0}");
    let mut ev = Evaluator::new();
    let f = env.lookup("get_x").unwrap().clone();
    let rec = Value::Record(vec![
        ("x".into(), Value::Int(42)),
        ("y".into(), Value::Int(0)),
    ]);
    assert_eq!(ev.apply(f, rec).unwrap(), Value::Int(42));
}

#[test]
fn record_field_in_function() {
    let env = run_ok("dist_sq p = p.x * p.x + p.y * p.y");
    let mut ev = Evaluator::new();
    let f = env.lookup("dist_sq").unwrap().clone();
    let p = Value::Record(vec![
        ("x".into(), Value::Int(3)),
        ("y".into(), Value::Int(4)),
    ]);
    assert_eq!(ev.apply(f, p).unwrap(), Value::Int(25));
}

// ── Modules (Phase 9.5) ───────────────────────────────

#[test]
fn module_simple() {
    let src = "\
mod Math
  square x = x * x
use Math (square)
main = square 7";
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Int(49));
}

#[test]
fn module_constant() {
    let src = "\
mod Consts
  pi = 314
use Consts (pi)
main = pi";
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Int(314));
}

// ── Row Polymorphism (Phase 11.2) ─────────────────────

#[test]
fn row_poly_field_access() {
    // get_x accepts any record with an x field
    let env = run_ok("get_x r = r.x");
    let mut ev = Evaluator::new();
    let f = env.lookup("get_x").unwrap().clone();
    let r1 = Value::Record(vec![("x".into(), Value::Int(5)), ("y".into(), Value::Int(10))]);
    let r2 = Value::Record(vec![("x".into(), Value::Int(99)), ("z".into(), Value::Bool(true))]);
    assert_eq!(ev.apply(f.clone(), r1).unwrap(), Value::Int(5));
    assert_eq!(ev.apply(f, r2).unwrap(), Value::Int(99));
}

// ── Strings ───────────────────────────────────────────

#[test]
fn string_concat_op() {
    assert_eq!(ev("\"hello\" ++ \" world\""), Value::Str("hello world".into()));
}

#[test]
fn string_eq() {
    assert_eq!(ev("\"abc\" == \"abc\""), Value::Bool(true));
    assert_eq!(ev("\"abc\" == \"def\""), Value::Bool(false));
}

#[test]
fn string_show_int() {
    assert_eq!(ev("show 42"), Value::Str("42".into()));
}

// ── Compose ───────────────────────────────────────────

#[test]
fn compose_op() {
    let env = run_ok("double x = x * 2\nadd1 x = x + 1\nmain = (double >> add1) 5");
    match env.lookup("main") {
        Some(Value::Func { equations, .. }) => {
            let mut ev = Evaluator::new();
            let v = ev.eval(&env, &equations[0].body).unwrap();
            assert_eq!(v, Value::Int(11));
        }
        _ => panic!("main not found"),
    }
}

// ── Where bindings ────────────────────────────────────

#[test]
fn where_bindings_basic() {
    let src = "hyp a b = root\n  sq_sum = a * a + b * b\n  root = sq_sum";
    let env = run_ok(src);
    let mut ev = Evaluator::new();
    let f = env.lookup("hyp").unwrap().clone();
    let partial = ev.apply(f, Value::Int(3)).unwrap();
    let result = ev.apply(partial, Value::Int(4)).unwrap();
    assert_eq!(result, Value::Int(25));
}

// ── Full Programs (extended) ──────────────────────────

#[test]
fn full_fibonacci() {
    let src = "fib 0 = 0\nfib 1 = 1\nfib n = fib (n - 1) + fib (n - 2)";
    let env = run_ok(src);
    let mut ev = Evaluator::new();
    let f = env.lookup("fib").unwrap().clone();
    assert_eq!(ev.apply(f.clone(), Value::Int(0)).unwrap(), Value::Int(0));
    assert_eq!(ev.apply(f.clone(), Value::Int(1)).unwrap(), Value::Int(1));
    assert_eq!(ev.apply(f.clone(), Value::Int(7)).unwrap(), Value::Int(13));
    assert_eq!(ev.apply(f, Value::Int(10)).unwrap(), Value::Int(55));
}

#[test]
fn full_adt_pattern_matching() {
    let src = "\
Maybe a = Just a | None
fromJust (Just v) = v
fromJust None = 0";
    let env = run_ok(src);
    let mut ev = Evaluator::new();
    let f = env.lookup("fromJust").unwrap().clone();
    let just42 = Value::Con("Just".into(), vec![Value::Int(42)]);
    let none = Value::Con("None".into(), vec![]);
    assert_eq!(ev.apply(f.clone(), just42).unwrap(), Value::Int(42));
    assert_eq!(ev.apply(f, none).unwrap(), Value::Int(0));
}

#[test]
fn full_higher_order_foldl() {
    // User-defined foldl: foldl f acc [1 2 3 4 5] = 15
    let src = "\
myfoldl f acc [] = acc
myfoldl f acc (x:xs) = myfoldl f (f acc x) xs
result = myfoldl (\\acc x -> acc + x) 0 [1 2 3 4 5]";
    let env = run_ok(src);
    match env.lookup("result") {
        Some(Value::Func { equations, .. }) => {
            let mut ev = Evaluator::new();
            let v = ev.eval(&env, &equations[0].body).unwrap();
            assert_eq!(v, Value::Int(15));
        }
        _ => panic!("result not found"),
    }
}

// ── Type classes (Phase 12) ───────────────────────────

#[test]
fn typeclass_show_adt() {
    // impl Show on a custom ADT; `show` dispatches to the impl method
    let src = "\
Color = Red | Green | Blue
trait Show a
  show : a -> String
impl Show Color
  show Red = \"Red\"
  show Green = \"Green\"
  show Blue = \"Blue\"
main = show Red";
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Str("Red".into()));
}

#[test]
fn typeclass_impl_fallback_to_existing() {
    // impl equations prepend to the existing user-defined function,
    // so non-matching values fall through to the existing equation
    let src = "\
Color = Red | Green
show_color Red = \"Red\"
show_color Green = \"Green\"
show_color _ = \"Other\"
impl ShowColor Color
  show_color Red = \"Red\"
  show_color Green = \"Green\"
main = show_color Green";
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Str("Green".into()));
}

// ── Phase 13: Type class improvements ─────────────────

// Eq for floats
#[test]
fn float_eq_true() {
    assert_eq!(ev("1.5 == 1.5"), Value::Bool(true));
}

#[test]
fn float_eq_false() {
    assert_eq!(ev("1.5 == 2.0"), Value::Bool(false));
}

#[test]
fn float_neq() {
    assert_eq!(ev("1.5 != 2.0"), Value::Bool(true));
}

// Ord for floats
#[test]
fn float_lt_true() {
    assert_eq!(ev("1.5 < 2.0"), Value::Bool(true));
}

#[test]
fn float_lt_false() {
    assert_eq!(ev("2.0 < 1.5"), Value::Bool(false));
}

#[test]
fn float_gt_true() {
    assert_eq!(ev("3.0 > 1.5"), Value::Bool(true));
}

#[test]
fn float_gt_false() {
    assert_eq!(ev("1.0 > 3.0"), Value::Bool(false));
}

#[test]
fn float_lte_equal() {
    assert_eq!(ev("1.0 <= 1.0"), Value::Bool(true));
}

#[test]
fn float_lte_less() {
    assert_eq!(ev("1.0 <= 2.0"), Value::Bool(true));
}

#[test]
fn float_gte_equal() {
    assert_eq!(ev("2.5 >= 2.5"), Value::Bool(true));
}

#[test]
fn float_gte_greater() {
    assert_eq!(ev("3.0 >= 1.5"), Value::Bool(true));
}

// Eq for records
#[test]
fn record_eq_true() {
    assert_eq!(ev("{x = 1} == {x = 1}"), Value::Bool(true));
}

#[test]
fn record_eq_false() {
    assert_eq!(ev("{x = 1} == {x = 2}"), Value::Bool(false));
}

// Eq for lists
#[test]
fn list_eq_true() {
    assert_eq!(ev("[1 2 3] == [1 2 3]"), Value::Bool(true));
}

#[test]
fn list_eq_false() {
    assert_eq!(ev("[1 2] == [1 3]"), Value::Bool(false));
}

// show for floats
#[test]
fn show_float_fractional() {
    let src = "main = show 3.14";
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Str("3.14".into()));
}

#[test]
fn show_float_whole() {
    // whole-number float displays with one decimal place: "3.0"
    let src = "main = show 3.0";
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Str("3.0".into()));
}

#[test]
fn show_float_half() {
    let src = "main = show 0.5";
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Str("0.5".into()));
}

// Comprehensive type class test: ADT with float fields
#[test]
fn typeclass_adt_float_eval() {
    let src = "\
Expr = Num Float | Add Expr Expr
eval_expr (Num x) = x
eval_expr (Add a b) = eval_expr a + eval_expr b
main = eval_expr (Add (Num 1.5) (Num 2.5))";
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Float(4.0));
}
