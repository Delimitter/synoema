// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

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

// ── Power operator ────────────────────────────────────

#[test]
fn pow_int() {
    assert_eq!(ev("2 ** 10"), Value::Int(1024));
}

#[test]
fn pow_int_zero() {
    assert_eq!(ev("5 ** 0"), Value::Int(1));
}

#[test]
fn pow_int_one() {
    assert_eq!(ev("7 ** 1"), Value::Int(7));
}

#[test]
fn pow_float() {
    assert_eq!(ev("2.0 ** 3.0"), Value::Float(8.0));
}

#[test]
fn pow_right_assoc() {
    // 2 ** 3 ** 2 should be 2 ** (3 ** 2) = 2 ** 9 = 512
    assert_eq!(ev("2 ** 3 ** 2"), Value::Int(512));
}

// ── Float math builtins ───────────────────────────────

#[test]
fn sqrt_float() {
    assert_eq!(ev("sqrt 4.0"), Value::Float(2.0));
}

#[test]
fn sqrt_nine() {
    assert_eq!(ev("sqrt 9.0"), Value::Float(3.0));
}

#[test]
fn abs_neg_int() {
    assert_eq!(ev("abs (0 - 5)"), Value::Int(5));
}

#[test]
fn abs_pos_int() {
    assert_eq!(ev("abs 5"), Value::Int(5));
}

#[test]
fn abs_float() {
    assert_eq!(ev("abs (0.0 - 3.5)"), Value::Float(3.5));
}

#[test]
fn floor_float() {
    assert_eq!(ev("floor 3.7"), Value::Float(3.0));
}

#[test]
fn floor_neg_float() {
    assert_eq!(ev("floor (0.0 - 3.2)"), Value::Float(-4.0));
}

#[test]
fn ceil_float() {
    assert_eq!(ev("ceil 3.2"), Value::Float(4.0));
}

#[test]
fn round_float_up() {
    assert_eq!(ev("round 3.6"), Value::Float(4.0));
}

#[test]
fn round_float_down() {
    assert_eq!(ev("round 3.2"), Value::Float(3.0));
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

// ── Phase 14: IO / Effects ────────────────────────────

#[test]
fn unit_literal() {
    assert_eq!(ev("()"), Value::Unit);
}

#[test]
fn print_returns_unit() {
    let (val, output) = run_main("main = print 42");
    assert_eq!(val, Value::Unit);
    assert_eq!(output, vec!["42"]);
}

#[test]
fn print_string() {
    let (_, output) = run_main(r#"main = print "hello""#);
    assert_eq!(output, vec!["hello"]);
}

#[test]
fn seq_two_prints() {
    let (val, output) = run_main(r#"main = print "hello" ; print "world""#);
    assert_eq!(val, Value::Unit);
    assert_eq!(output, vec!["hello", "world"]);
}

#[test]
fn seq_returns_right() {
    assert_eq!(ev("42 ; 99"), Value::Int(99));
}

#[test]
fn seq_discards_left() {
    // Left side evaluated (side effects), right side returned
    assert_eq!(ev("() ; true"), Value::Bool(true));
}

#[test]
fn seq_chain_three_prints() {
    let (_, output) = run_main(
        r#"main = print "a" ; print "b" ; print "c""#
    );
    assert_eq!(output, vec!["a", "b", "c"]);
}

#[test]
fn print_any_type() {
    let (_, output) = run_main("main = print true");
    assert_eq!(output, vec!["true"]);
}

#[test]
fn print_list() {
    let (_, output) = run_main("main = print [1 2 3]");
    assert_eq!(output, vec!["[1 2 3]"]);
}

#[test]
fn seq_with_pipe() {
    // print "hello" ; [1 2 3] |> print
    let (_, output) = run_main(
        r#"main = print "hello" ; [1 2 3] |> print"#
    );
    assert_eq!(output, vec!["hello", "[1 2 3]"]);
}

#[test]
fn unit_eq() {
    assert_eq!(ev("() == ()"), Value::Bool(true));
}

// ── Float arithmetic through full pipeline (run_main) ─

#[test]
fn run_float_add() {
    let (val, _) = run_main("main = 3.14 + 2.71");
    assert_eq!(val, Value::Float(5.85));
}

#[test]
fn run_float_sub() {
    let (val, _) = run_main("main = 5.0 - 1.5");
    assert_eq!(val, Value::Float(3.5));
}

#[test]
fn run_float_mul() {
    let (val, _) = run_main("main = 2.0 * 3.0");
    assert_eq!(val, Value::Float(6.0));
}

#[test]
fn run_float_div() {
    let (val, _) = run_main("main = 10.0 / 4.0");
    assert_eq!(val, Value::Float(2.5));
}

#[test]
fn run_float_pow() {
    let (val, _) = run_main("main = 2.0 ** 3.0");
    assert_eq!(val, Value::Float(8.0));
}

#[test]
fn run_float_with_sqrt() {
    let (val, _) = run_main("main = sqrt 4.0 + 1.0");
    assert_eq!(val, Value::Float(3.0));
}

// ── String concat through full pipeline ───────────────

#[test]
fn run_string_concat() {
    let (val, _) = run_main("main = \"hello\" ++ \" world\"");
    assert_eq!(val, Value::Str("hello world".into()));
}

// ── Division by zero — mixed types ────────────────────

#[test]
fn div_int_by_float_zero() {
    let result = eval_expr("5 / 0.0");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("Division by zero"));
}

#[test]
fn div_float_by_int_zero() {
    let result = eval_expr("5.0 / 0");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("Division by zero"));
}

// ── Power overflow protection ─────────────────────────

#[test]
fn pow_int_overflow() {
    let result = eval_expr("10 ** 20");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("overflow"));
}

#[test]
fn pow_int_normal() {
    assert_eq!(ev("2 ** 10"), Value::Int(1024));
}

// ── Logic (extended) ──────────────────────────────────────────

#[test]
fn or_false_false() { assert_eq!(ev("false || false"), Value::Bool(false)); }

#[test]
fn not_builtin() {
    // `not` is a builtin in the evaluator
    assert_eq!(ev("not true"), Value::Bool(false));
    assert_eq!(ev("not false"), Value::Bool(true));
}

// ── Strings (extended) ────────────────────────────────────────

#[test]
fn string_neq() {
    assert_eq!(ev("\"abc\" != \"def\""), Value::Bool(true));
    assert_eq!(ev("\"abc\" != \"abc\""), Value::Bool(false));
}

#[test]
fn string_length_builtin() {
    assert_eq!(ev("length \"hello\""), Value::Int(5));
    assert_eq!(ev("length \"\""), Value::Int(0));
}

#[test]
fn show_bool() {
    let (val, _) = run_main("main = show true");
    assert_eq!(val, Value::Str("true".into()));
    let (val2, _) = run_main("main = show false");
    assert_eq!(val2, Value::Str("false".into()));
}

#[test]
fn show_list_int() {
    let (val, _) = run_main("main = show [1 2 3]");
    assert_eq!(val, Value::Str("[1 2 3]".into()));
}

#[test]
fn show_in_string_concat() {
    let (val, _) = run_main(r#"main = "n=" ++ show 42"#);
    assert_eq!(val, Value::Str("n=42".into()));
}

// ── List (extended) ───────────────────────────────────────────

#[test]
fn list_comp_double_transform() {
    let (val, _) = run_main("main = [x * 2 | x <- [1..5]]");
    assert_eq!(val, Value::List(vec![
        Value::Int(2), Value::Int(4), Value::Int(6), Value::Int(8), Value::Int(10)
    ]));
}

#[test]
fn cons_builds_list() {
    let (val, _) = run_main("main = 1 : 2 : 3 : []");
    assert_eq!(val, Value::List(vec![
        Value::Int(1), Value::Int(2), Value::Int(3)
    ]));
}

// ── Full Programs (extended) ──────────────────────────────────

#[test]
fn full_gcd() {
    let src = "gcd a 0 = a\ngcd a b = gcd b (a % b)";
    let env = run_ok(src);
    let mut ev = Evaluator::new();
    let f = env.lookup("gcd").unwrap().clone();
    let partial = ev.apply(f, Value::Int(48)).unwrap();
    assert_eq!(ev.apply(partial, Value::Int(18)).unwrap(), Value::Int(6));
}

// ── Records (extended) ────────────────────────────────────────

#[test]
fn record_three_fields() {
    let val = ev("{a = 10, b = 20, c = 30}");
    assert_eq!(val, Value::Record(vec![
        ("a".into(), Value::Int(10)),
        ("b".into(), Value::Int(20)),
        ("c".into(), Value::Int(30)),
    ]));
}

#[test]
fn record_float_field() {
    let env = run_ok("p = {pi = 3.14, r = 2.0}\nmain = p.pi");
    match env.lookup("main") {
        Some(Value::Func { equations, .. }) => {
            let mut ev = Evaluator::new();
            let v = ev.eval(&env, &equations[0].body).unwrap();
            assert_eq!(v, Value::Float(3.14));
        }
        _ => panic!("main not found"),
    }
}

// ── Pipe (extended) ───────────────────────────────────────────

#[test]
fn pipe_chain_two_steps() {
    let env = run_ok("double x = x * 2\nresult = 5 |> double |> double");
    match env.lookup("result") {
        Some(Value::Func { equations, .. }) => {
            let mut ev = Evaluator::new();
            let v = ev.eval(&env, &equations[0].body).unwrap();
            assert_eq!(v, Value::Int(20));
        }
        _ => panic!("result not found"),
    }
}

// ── Modules (extended) ────────────────────────────────────────

#[test]
fn module_two_imports() {
    let src = "\
mod Math
  double x = x * 2
  square x = x * x
use Math (double square)
main = double (square 3)";
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Int(18));
}

// ── IO (extended) ─────────────────────────────────────────────

#[test]
fn print_float_output() {
    let (_, output) = run_main("main = print 3.14");
    assert_eq!(output, vec!["3.14"]);
}

// ── Builtins (extended) ───────────────────────────────────────

#[test]
fn filter_builtin() {
    assert_eq!(
        ev("filter even [1 2 3 4 5 6]"),
        Value::List(vec![Value::Int(2), Value::Int(4), Value::Int(6)])
    );
}

#[test]
fn map_builtin() {
    assert_eq!(
        ev("map (\\x -> x * 2) [1 2 3]"),
        Value::List(vec![Value::Int(2), Value::Int(4), Value::Int(6)])
    );
}

#[test]
fn foldl_builtin() {
    assert_eq!(
        ev("foldl (\\acc x -> acc + x) 0 [1 2 3 4 5]"),
        Value::Int(15)
    );
}

// ── Phase 18: String stdlib ───────────────────────────────────────────────────

#[test]
fn str_len_basic() {
    assert_eq!(ev("str_len \"hello\""), Value::Int(5));
    assert_eq!(ev("str_len \"\""), Value::Int(0));
}

#[test]
fn str_slice_basic() {
    assert_eq!(ev("str_slice \"hello world\" 0 5"), Value::Str("hello".into()));
    assert_eq!(ev("str_slice \"hello world\" 6 11"), Value::Str("world".into()));
}

#[test]
fn str_slice_out_of_bounds() {
    // Clamps to string length
    assert_eq!(ev("str_slice \"hi\" 0 100"), Value::Str("hi".into()));
    assert_eq!(ev("str_slice \"hi\" 5 10"), Value::Str("".into()));
}

#[test]
fn str_find_found() {
    assert_eq!(ev("str_find \"hello world\" \" \" 0"), Value::Int(5));
    assert_eq!(ev("str_find \"abcabc\" \"bc\" 0"), Value::Int(1));
}

#[test]
fn str_find_not_found() {
    assert_eq!(ev("str_find \"hello\" \"xyz\" 0"), Value::Int(-1));
}

#[test]
fn str_find_from_offset() {
    // Search from position 2 — skips first "bc"
    assert_eq!(ev("str_find \"abcabc\" \"bc\" 2"), Value::Int(4));
}

#[test]
fn str_find_empty_needle() {
    // Empty needle returns `from` position
    assert_eq!(ev("str_find \"hello\" \"\" 3"), Value::Int(3));
}

#[test]
fn str_starts_with_true() {
    assert_eq!(ev("str_starts_with \"hello\" \"hel\""), Value::Bool(true));
    assert_eq!(ev("str_starts_with \"hello\" \"\""), Value::Bool(true));
}

#[test]
fn str_starts_with_false() {
    assert_eq!(ev("str_starts_with \"hello\" \"world\""), Value::Bool(false));
}

#[test]
fn str_trim_basic() {
    assert_eq!(ev("str_trim \"  hello  \""), Value::Str("hello".into()));
    assert_eq!(ev("str_trim \"\\thello\\n\""), Value::Str("hello".into()));
    assert_eq!(ev("str_trim \"nospace\""), Value::Str("nospace".into()));
}

// ── Phase 18: json_escape ─────────────────────────────────────────────────────

#[test]
fn json_escape_quotes() {
    assert_eq!(
        ev("json_escape \"say \\\"hello\\\"\""),
        Value::Str("say \\\"hello\\\"".into())
    );
}

#[test]
fn json_escape_newline() {
    assert_eq!(
        ev("json_escape \"line1\\nline2\""),
        Value::Str("line1\\nline2".into())
    );
}

#[test]
fn json_escape_backslash() {
    assert_eq!(
        ev("json_escape \"a\\\\b\""),
        Value::Str("a\\\\b".into())
    );
}

#[test]
fn json_escape_plain() {
    assert_eq!(ev("json_escape \"hello\""), Value::Str("hello".into()));
}

// ── Phase 18: str builtins in programs ────────────────────────────────────────

#[test]
fn str_parse_path() {
    // Replicate parse_path from stress_server.sno
    let src = r#"
parse_path req =
  sp1 = str_find req " " 0
  sp2 = str_find req " " (sp1 + 1)
  ? sp1 < 0 -> "/" : ? sp2 < 0 -> str_slice req (sp1 + 1) (str_len req) : str_slice req (sp1 + 1) sp2
main = parse_path "GET /run/lexer HTTP/1.0"
"#;
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Str("/run/lexer".into()));
}

#[test]
fn str_suite_extraction() {
    let src = r#"
parse_suite path =
  raw  = str_slice path 5 (str_len path)
  qpos = str_find raw "?" 0
  ? qpos < 0 -> raw : str_slice raw 0 qpos
main = parse_suite "/run/lexer"
"#;
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Str("lexer".into()));
}

#[test]
fn str_suite_with_query() {
    let src = r#"
parse_suite path =
  raw  = str_slice path 5 (str_len path)
  qpos = str_find raw "?" 0
  ? qpos < 0 -> raw : str_slice raw 0 qpos
main = parse_suite "/run/lexer?slow=1"
"#;
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Str("lexer".into()));
}

#[test]
fn str_sse_data_format() {
    let src = r#"
sse_data text = "data: {\"line\":\"" ++ json_escape text ++ "\"}\n\n"
main = sse_data "hello world"
"#;
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Str("data: {\"line\":\"hello world\"}\n\n".into()));
}

// ── Phase 18: file_read ───────────────────────────────────────────────────────

#[test]
fn file_read_existing() {
    // Read Cargo.toml (always exists in workspace root, which is the test cwd)
    let src = "main = str_len (file_read \"Cargo.toml\")";
    let (val, _) = run_main(src);
    // File exists and is non-empty
    match val {
        Value::Int(n) => assert!(n > 10, "Expected non-trivial file size, got {}", n),
        _ => panic!("Expected Int, got {:?}", val),
    }
}

// ── Phase B: scope / spawn ────────────────────────────────────────────────────

#[test]
fn scope_returns_result() {
    // scope { expr } returns the value of expr
    let (val, _) = run_main("main = scope { 42 }");
    assert_eq!(val, Value::Int(42));
}

#[test]
fn scope_unit_result() {
    // scope with only spawn expressions returns Unit
    let src = "
f x = x
main = scope { spawn (f 1) }
";
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Unit);
}

#[test]
fn scope_sequential_seq() {
    // scope { e1 ; e2 } returns e2
    let (val, _) = run_main("main = scope { 10 ; 20 }");
    assert_eq!(val, Value::Int(20));
}

#[test]
fn scope_with_spawn_runs() {
    // spawn executes side effects; scope waits for completion
    let src = r#"
f x = print (show x)
main = scope { spawn (f 1) ; spawn (f 2) }
"#;
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Unit);
}

#[test]
fn scope_nested() {
    // Nested scopes work correctly
    let src = "
f x = x * 2
main = scope { scope { spawn (f 3) } ; 99 }
";
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Int(99));
}

#[test]
fn spawn_returns_unit() {
    let src = "
f x = x
main = scope { spawn (f 5) }
";
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Unit);
}

#[test]
fn scope_spawn_with_channel() {
    // Producer spawns, main thread receives — real concurrency test
    let src = r#"
producer ch = send ch 42

main =
  ch = chan
  scope {
    spawn (producer ch)
    recv ch
  }
"#;
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Int(42));
}

#[test]
fn scope_spawn_multiple_sends() {
    // Multiple sends, one recv per message
    let src = r#"
producer ch =
  send ch 10
  send ch 20
  send ch 30

main =
  ch = chan
  scope {
    spawn (producer ch)
    a = recv ch
    b = recv ch
    c = recv ch
    a + b + c
  }
"#;
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Int(60));
}

#[test]
fn file_read_missing_is_err() {
    let result = eval_main("main = file_read \"/nonexistent/path/xyz.txt\"");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("file_read"));
}

// ── Memory Management v2: fd_open / fd_open_write ─────────────────────────────

#[test]
fn fd_open_read_line() {
    // fd_open opens a file for reading, fd_readline reads one line
    let src = r#"
main =
  fd = fd_open "Cargo.toml"
  line = fd_readline fd
  fd_close fd |> \_ -> str_len line
"#;
    let (val, _) = run_main(src);
    match val {
        Value::Int(n) => assert!(n > 0, "Expected non-empty first line, got {}", n),
        _ => panic!("Expected Int, got {:?}", val),
    }
}

#[test]
fn fd_open_read_multiple_lines() {
    // fd_readline reads successive lines from the same fd
    let src = r#"
main =
  fd = fd_open "Cargo.toml"
  l1 = fd_readline fd
  l2 = fd_readline fd
  fd_close fd |> \_ -> l1 ++ "|" ++ l2
"#;
    let (val, _) = run_main(src);
    match &val {
        Value::Str(s) => {
            assert!(s.contains("|"), "Expected two lines joined by |, got {:?}", s);
            let parts: Vec<&str> = s.split('|').collect();
            assert_eq!(parts.len(), 2);
            assert!(!parts[0].is_empty());
        }
        _ => panic!("Expected Str, got {:?}", val),
    }
}

#[test]
fn fd_open_missing_file_is_err() {
    let result = eval_main(r#"main = fd_open "/nonexistent/path/xyz.txt""#);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("fd_open"));
}

#[test]
fn fd_open_write_creates_file() {
    use std::io::Write;
    // Create a temp file path
    let tmp = std::env::temp_dir().join("synoema_test_fd_open_write.txt");
    let tmp_path = tmp.to_str().unwrap().replace('\\', "/");
    let src = format!(r#"
main =
  fd = fd_open_write "{}"
  fd_write fd "hello from synoema"
  fd_close fd |> \_ -> 42
"#, tmp_path);
    let (val, _) = run_main(&src);
    assert_eq!(val, Value::Int(42));
    // Verify file contents
    let contents = std::fs::read_to_string(&tmp).unwrap();
    assert_eq!(contents, "hello from synoema");
    // Cleanup
    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn fd_open_write_then_read() {
    // Write a file, then read it back with fd_open
    let tmp = std::env::temp_dir().join("synoema_test_write_then_read.txt");
    let tmp_path = tmp.to_str().unwrap().replace('\\', "/");

    // Step 1: write the file
    let write_src = format!(r#"
main =
  wfd = fd_open_write "{}"
  fd_write wfd "line one"
  fd_close wfd |> \_ -> 1
"#, tmp_path);
    let (val, _) = run_main(&write_src);
    assert_eq!(val, Value::Int(1));

    // Step 2: read it back
    let read_src = format!(r#"
main =
  rfd = fd_open "{}"
  line = fd_readline rfd
  fd_close rfd |> \_ -> line
"#, tmp_path);
    let (val, _) = run_main(&read_src);
    assert_eq!(val, Value::Str("line one".to_string()));

    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn fd_open_write_missing_dir_is_err() {
    let result = eval_main(r#"main = fd_open_write "/nonexistent/dir/file.txt""#);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("fd_open_write"));
}

// ── Phase C: chan / send / recv ────────────────────────────────────────────────

#[test]
fn chan_send_recv_basic() {
    // Same-thread send+recv (channel is buffered / async)
    let src = r#"
main =
  ch = chan
  send ch 99
  recv ch
"#;
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Int(99));
}

#[test]
fn chan_send_recv_string() {
    let src = r#"
main =
  ch = chan
  send ch "hello"
  recv ch
"#;
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Str("hello".into()));
}

#[test]
fn chan_multiple_values() {
    let src = r#"
main =
  ch = chan
  send ch 1
  send ch 2
  send ch 3
  a = recv ch
  b = recv ch
  c = recv ch
  a + b + c
"#;
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Int(6));
}

#[test]
fn chan_type_polymorphism() {
    // send/recv works for any type (Bool)
    let src = r#"
main =
  ch = chan
  send ch true
  recv ch
"#;
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Bool(true));
}

#[test]
fn chan_type_check_passes() {
    // Type checker accepts correctly typed code
    let src = r#"
main =
  ch = chan
  send ch 42
  recv ch
"#;
    let _ = run_main(src); // should not panic
}

#[test]
fn chan_in_scope_concurrent() {
    // Real producer-consumer concurrency
    let src = r#"
producer ch =
  send ch 100

main =
  ch = chan
  scope {
    spawn (producer ch)
    recv ch
  }
"#;
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Int(100));
}

#[test]
fn chan_two_producers() {
    let src = r#"
p1 ch = send ch 10
p2 ch = send ch 20

main =
  ch = chan
  scope {
    spawn (p1 ch)
    spawn (p2 ch)
    a = recv ch
    b = recv ch
    a + b
  }
"#;
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Int(30));
}

#[test]
fn chan_list_value() {
    // Channels can carry list values
    let src = r#"
main =
  ch = chan
  send ch [1 2 3]
  recv ch
"#;
    let (val, _) = run_main(src);
    assert_eq!(val, Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
}

// ── Type Aliases (e2e) ───────────────────────────────────

#[test]
fn type_alias_program_runs() {
    // Type alias should be transparent — program runs as if alias wasn't there
    let (val, _) = run_main("type Num = Int\nadd : Num -> Num -> Num\nadd x y = x + y\nmain = add 3 4");
    assert_eq!(val, Value::Int(7));
}

#[test]
fn type_alias_with_adt() {
    // Type alias + ADT — alias used in program, not blocking ADT definitions
    let (val, _) = run_main("type Num = Int\nMaybe a = Just a | None\nmain = Just 42");
    assert_eq!(val, Value::Con("Just".into(), vec![Value::Int(42)]));
}

// ── String Interpolation ─────────────────────────────────

#[test]
fn interp_simple_var() {
    let (val, _) = run_main("name = \"World\"\nmain = \"Hello ${name}\"");
    assert_eq!(val, Value::Str("Hello World".into()));
}

#[test]
fn interp_int_show() {
    let (val, _) = run_main("x = 42\nmain = \"x is ${x}\"");
    assert_eq!(val, Value::Str("x is 42".into()));
}

#[test]
fn interp_expression() {
    let (val, _) = run_main("main = \"sum is ${2 + 3}\"");
    assert_eq!(val, Value::Str("sum is 5".into()));
}

#[test]
fn interp_multiple() {
    let (val, _) = run_main("a = 1\nb = 2\nmain = \"${a} + ${b} = ${a + b}\"");
    assert_eq!(val, Value::Str("1 + 2 = 3".into()));
}

#[test]
fn interp_string_var_no_quotes() {
    // show "hello" should produce "hello" not "\"hello\""
    let (val, _) = run_main("s = \"world\"\nmain = \"hello ${s}\"");
    assert_eq!(val, Value::Str("hello world".into()));
}

#[test]
fn interp_escaped_dollar() {
    let val = ev(r#""\$ is money""#);
    assert_eq!(val, Value::Str("$ is money".into()));
}

#[test]
fn interp_no_interp_stays_plain() {
    let val = ev(r#""no interpolation""#);
    assert_eq!(val, Value::Str("no interpolation".into()));
}

#[test]
fn interp_only_expr() {
    let (val, _) = run_main("main = \"${42}\"");
    assert_eq!(val, Value::Str("42".into()));
}

// ── LLM Error Feedback Integration ─────────────────────

#[test]
fn error_enrichment_type_mismatch() {
    let err = run("main = 1 + true").unwrap_err();
    assert_eq!(err.code, "type_mismatch");
    assert!(err.llm_hint.is_some(), "type_mismatch should have llm_hint");
    assert_eq!(err.fixability, Some(synoema_diagnostic::Fixability::Trivial));
}

#[test]
fn error_enrichment_unbound_var() {
    let err = run("main = foo").unwrap_err();
    assert!(err.llm_hint.is_some(), "unbound_variable should have llm_hint");
    assert_eq!(err.fixability, Some(synoema_diagnostic::Fixability::Easy));
}

#[test]
fn error_enrichment_json_output() {
    let err = run("main = 1 + true").unwrap_err();
    let json = synoema_diagnostic::render_json(&err);
    assert!(json.contains("\"llm_hint\":"), "JSON should contain llm_hint");
    assert!(json.contains("\"fixability\":"), "JSON should contain fixability");
}

// ── derive(Show, Eq, Ord) ───────────────────────────────

#[test]
fn derive_show_enum() {
    let (val, _) = run_main("Color = Red | Green | Blue derive (Show)\nmain = show Red");
    assert_eq!(val, Value::Str("Red".into()));
}

#[test]
fn derive_show_with_fields() {
    let (val, _) = run_main("Maybe a = Just a | Nothing derive (Show)\nmain = show (Just 42)");
    assert_eq!(val, Value::Str("Just 42".into()));
}

#[test]
fn derive_show_nothing() {
    let (val, _) = run_main("Maybe a = Just a | Nothing derive (Show)\nmain = show Nothing");
    assert_eq!(val, Value::Str("Nothing".into()));
}

#[test]
fn derive_eq_same() {
    let (val, _) = run_main("trait Eq a\n  eq : a -> a -> Bool\nColor = Red | Green | Blue derive (Eq)\nmain = eq Red Red");
    assert_eq!(val, Value::Bool(true));
}

#[test]
fn derive_eq_diff() {
    let (val, _) = run_main("trait Eq a\n  eq : a -> a -> Bool\nColor = Red | Green | Blue derive (Eq)\nmain = eq Red Blue");
    assert_eq!(val, Value::Bool(false));
}

#[test]
fn derive_eq_with_fields_same() {
    let (val, _) = run_main("trait Eq a\n  eq : a -> a -> Bool\nMaybe a = Just a | Nothing derive (Eq)\nmain = eq (Just 1) (Just 1)");
    assert_eq!(val, Value::Bool(true));
}

#[test]
fn derive_eq_with_fields_diff() {
    let (val, _) = run_main("trait Eq a\n  eq : a -> a -> Bool\nMaybe a = Just a | Nothing derive (Eq)\nmain = eq (Just 1) (Just 2)");
    assert_eq!(val, Value::Bool(false));
}

#[test]
fn derive_eq_diff_constructors() {
    let (val, _) = run_main("trait Eq a\n  eq : a -> a -> Bool\nMaybe a = Just a | Nothing derive (Eq)\nmain = eq (Just 1) Nothing");
    assert_eq!(val, Value::Bool(false));
}

#[test]
fn derive_ord_less() {
    let (val, _) = run_main("trait Ord a\n  cmp : a -> a -> Int\nColor = Red | Green | Blue derive (Ord)\nmain = cmp Red Green");
    assert_eq!(val, Value::Int(-1));
}

#[test]
fn derive_ord_equal() {
    let (val, _) = run_main("trait Ord a\n  cmp : a -> a -> Int\nColor = Red | Green | Blue derive (Ord)\nmain = cmp Green Green");
    assert_eq!(val, Value::Int(0));
}

#[test]
fn derive_ord_greater() {
    let (val, _) = run_main("trait Ord a\n  cmp : a -> a -> Int\nColor = Red | Green | Blue derive (Ord)\nmain = cmp Blue Red");
    assert_eq!(val, Value::Int(1));
}

#[test]
fn derive_ord_first_vs_last() {
    let (val, _) = run_main("trait Ord a\n  cmp : a -> a -> Int\nColor = Red | Green | Blue derive (Ord)\nmain = cmp Red Blue");
    assert_eq!(val, Value::Int(-1));
}

#[test]
fn derive_ord_last_vs_middle() {
    let (val, _) = run_main("trait Ord a\n  cmp : a -> a -> Int\nColor = Red | Green | Blue derive (Ord)\nmain = cmp Blue Green");
    assert_eq!(val, Value::Int(1));
}

#[test]
fn derive_multiple_traits() {
    let (val, _) = run_main("trait Eq a\n  eq : a -> a -> Bool\ntrait Ord a\n  cmp : a -> a -> Int\nColor = Red | Green | Blue derive (Show, Eq, Ord)\nmain = show Red ++ \" \" ++ show (eq Red Red) ++ \" \" ++ show (cmp Red Green)");
    assert_eq!(val, Value::Str("Red true -1".into()));
}

#[test]
fn derive_manual_override_show() {
    // Manual impl should override derive (derive Show is a no-op, manual impl works)
    let (val, _) = run_main("Color = Red | Green | Blue derive (Show)\nimpl Show Color\n  show Red = \"RED\"\n  show Green = \"GREEN\"\n  show Blue = \"BLUE\"\nmain = show Red");
    assert_eq!(val, Value::Str("RED".into()));
}

#[test]
fn derive_unknown_trait_error() {
    let result = run("Color = Red | Green | Blue derive (Functor)\nmain = show Red");
    assert!(result.is_err());
}

#[test]
fn derive_recursive_adt() {
    let (val, _) = run_main("trait Eq a\n  eq : a -> a -> Bool\nList a = Cons a (List a) | Nil derive (Show, Eq)\nmain = show (Cons 1 (Cons 2 Nil))");
    assert_eq!(val, Value::Str("Cons 1 (Cons 2 Nil)".into()));
}

// ── Record Punning ──────────────────────────────────────────

#[test]
fn record_punning_basic() {
    let (val, _) = run_main("main =\n  x = 3\n  y = 4\n  r = {x, y}\n  r.x + r.y");
    assert_eq!(val, Value::Int(7));
}

#[test]
fn record_punning_mixed() {
    let (val, _) = run_main("main =\n  x = 10\n  r = {x, y = 20}\n  r.x + r.y");
    assert_eq!(val, Value::Int(30));
}

#[test]
fn record_pattern_punning() {
    let (val, _) = run_main("get_sum {x, y} = x + y\nmain = get_sum {x = 3, y = 4}");
    assert_eq!(val, Value::Int(7));
}

// ── Wildcard Import ─────────────────────────────────────────

#[test]
fn wildcard_import_basic() {
    let src = "\
mod Math
  square x = x * x
  cube x = x * x * x
use Math (*)
main = square 5 + cube 2";
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Int(33));
}

#[test]
fn wildcard_import_constant() {
    let src = "\
mod Consts
  pi = 314
  e = 271
use Consts (*)
main = pi + e";
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Int(585));
}

#[test]
fn wildcard_import_with_args() {
    let src = "\
mod Vec
  make x y = {x = x, y = y}
  dot a b = a.x * b.x + a.y * b.y
use Vec (*)
main = dot (make 3 4) (make 1 2)";
    let (val, _) = run_main(src);
    assert_eq!(val, Value::Int(11));
}
