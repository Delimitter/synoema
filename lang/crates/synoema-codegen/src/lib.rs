//! # synoema-codegen
//! Cranelift-based native code generator for the Synoema programming language.
//!
//! Compiles Core IR to native machine code via JIT.

pub mod compiler;
pub mod runtime;
pub use compiler::{Compiler, CompileError};
pub use runtime::arena_reset;

/// Parse, desugar, and JIT-compile an Synoema program, returning main() result as i64.
/// Strings are returned as tagged i64 pointers (use `display_result` for human-readable output).
pub fn compile_and_run(source: &str) -> Result<i64, String> {
    let program = synoema_parser::parse(source)
        .map_err(|e| format!("Parse error: {}", e))?;
    let program = synoema_types::resolve_modules(program);
    let core = synoema_core::desugar_program(&program);
    let core = synoema_core::optimize_program(core);
    let mut compiler = Compiler::new()
        .map_err(|e| format!("{}", e))?;
    let result = compiler.compile_and_run(&core)
        .map_err(|e| format!("{}", e));
    crate::runtime::arena_reset(); // Free all heap allocations from this run
    result
}

/// Parse, desugar, JIT-compile and return main() result as a display string.
/// Handles both integer results and tagged string results.
pub fn compile_and_display(source: &str) -> Result<String, String> {
    let result = compile_and_run(source)?;
    Ok(runtime::display_value(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jit(src: &str) -> i64 {
        compile_and_run(src)
            .unwrap_or_else(|e| panic!("Compile failed for:\n{}\nError: {}", src, e))
    }

    #[test]
    fn constant() { assert_eq!(jit("main = 42"), 42); }

    #[test]
    fn addition() { assert_eq!(jit("main = 10 + 32"), 42); }

    #[test]
    fn subtraction() { assert_eq!(jit("main = 50 - 8"), 42); }

    #[test]
    fn multiplication() { assert_eq!(jit("main = 6 * 7"), 42); }

    #[test]
    fn division() { assert_eq!(jit("main = 84 / 2"), 42); }

    #[test]
    fn modulo() { assert_eq!(jit("main = 47 % 5"), 2); }

    #[test]
    fn negation() { assert_eq!(jit("main = -(0 - 42)"), 42); }

    #[test]
    fn comparison_eq() { assert_eq!(jit("main = ? 1 == 1 -> 42 : 0"), 42); }

    #[test]
    fn comparison_lt() { assert_eq!(jit("main = ? 1 < 2 -> 42 : 0"), 42); }

    #[test]
    fn cond_true() { assert_eq!(jit("main = ? true -> 42 : 0"), 42); }

    #[test]
    fn cond_false() { assert_eq!(jit("main = ? false -> 0 : 42"), 42); }

    #[test]
    fn let_binding() { assert_eq!(jit("main =\n  x = 40\n  y = 2\n  x + y"), 42); }

    #[test]
    fn simple_function() { assert_eq!(jit("double x = x * 2\nmain = double 21"), 42); }

    #[test]
    fn two_arg_function() { assert_eq!(jit("add x y = x + y\nmain = add 40 2"), 42); }

    #[test]
    fn recursive_factorial() {
        assert_eq!(jit("fac 0 = 1\nfac n = n * fac (n - 1)\nmain = fac 10"), 3628800);
    }

    #[test]
    fn fibonacci() {
        assert_eq!(jit("fib 0 = 0\nfib 1 = 1\nfib n = fib (n - 1) + fib (n - 2)\nmain = fib 10"), 55);
    }

    #[test]
    fn ackermann() {
        assert_eq!(jit("ack 0 n = n + 1\nack m 0 = ack (m - 1) 1\nack m n = ack (m - 1) (ack m (n - 1))\nmain = ack 3 2"), 29);
    }

    #[test]
    fn nested_cond() {
        assert_eq!(jit("abs x = ? x < 0 -> 0 - x : x\nmain = abs (0 - 42)"), 42);
    }

    #[test]
    fn multi_function() {
        assert_eq!(jit("double x = x * 2\ninc x = x + 1\nmain = double (inc 20)"), 42);
    }

    // ── List Tests ──────────────────────────────────────

    #[test]
    fn list_length() {
        assert_eq!(jit("main = length [1 2 3]"), 3);
    }

    #[test]
    fn list_length_empty() {
        assert_eq!(jit("main = length []"), 0);
    }

    #[test]
    fn list_sum() {
        assert_eq!(jit("main = sum [10 20 12]"), 42);
    }

    #[test]
    fn list_head() {
        assert_eq!(jit("main = head [42 2 3]"), 42);
    }

    #[test]
    fn list_cons_head() {
        // Cons 42 onto empty list, take head
        assert_eq!(jit("main = head (42 : [])"), 42);
    }

    #[test]
    fn list_concat_sum() {
        // [1 2] ++ [3 4] → sum = 10
        assert_eq!(jit("main = sum ([1 2] ++ [3 4])"), 10);
    }

    #[test]
    fn list_pattern_sum() {
        // Recursive sum via pattern matching
        assert_eq!(jit("mysum [] = 0\nmysum (x:xs) = x + mysum xs\nmain = mysum [10 20 12]"), 42);
    }

    #[test]
    fn list_pattern_length() {
        assert_eq!(jit("mylen [] = 0\nmylen (_:xs) = 1 + mylen xs\nmain = mylen [1 2 3 4 5]"), 5);
    }

    #[test]
    fn list_reverse_length() {
        // Reverse via accumulator, check length
        assert_eq!(jit("rev [] = []\nrev (x:xs) = rev xs ++ [x]\nmain = length (rev [1 2 3 4 5])"), 5);
    }

    #[test]
    fn list_reverse_sum() {
        // Reverse preserves sum
        assert_eq!(jit("rev [] = []\nrev (x:xs) = rev xs ++ [x]\nmain = sum (rev [10 20 12])"), 42);
    }

    #[test]
    fn list_take() {
        assert_eq!(jit("take 0 xs = []\ntake n (x:xs) = x : take (n - 1) xs\nmain = sum (take 3 [10 20 30 40 50])"), 60);
    }

    #[test]
    fn list_drop() {
        assert_eq!(jit("drop 0 xs = xs\ndrop n (x:xs) = drop (n - 1) xs\nmain = sum (drop 2 [1 2 10 20 12])"), 42);
    }

    #[test]
    fn list_nested_pattern() {
        // Sum pairs: [[1,2],[3,4]] - check first pair sum
        assert_eq!(jit("first (x:xs) = x\nmain = first [42 1 2]"), 42);
    }

    #[test]
    fn list_singleton() {
        assert_eq!(jit("main = sum [42]"), 42);
    }

    #[test]
    fn list_cons_chain() {
        // Build list manually with cons, check sum
        assert_eq!(jit("main = sum (1 : 2 : 3 : [])"), 6);
    }

    #[test]
    fn list_concat_three() {
        assert_eq!(jit("main = sum ([1 2] ++ [3] ++ [4 5])"), 15);
    }

    // ── Closure / Higher-Order Function Tests ───────────────

    #[test]
    fn closure_identity() {
        // Simple lambda applied immediately
        assert_eq!(jit("main = (\\x -> x) 42"), 42);
    }

    #[test]
    fn closure_double() {
        // Lambda as a named def, applied to arg
        assert_eq!(jit("double = \\x -> x * 2\nmain = double 21"), 42);
    }

    #[test]
    fn closure_map_sum() {
        // map (\x -> x * 2) [1 2 3] = [2 4 6], sum = 12
        assert_eq!(jit(
            "map f [] = []\n\
             map f (x:xs) = f x : map f xs\n\
             main = sum (map (\\x -> x * 2) [1 2 3])"
        ), 12);
    }

    #[test]
    fn closure_filter_length() {
        // filter (\x -> x > 3) [1 2 3 4 5] = [4 5], length = 2
        assert_eq!(jit(
            "filter f [] = []\n\
             filter f (x:xs) = ? (f x) -> (x : filter f xs) : (filter f xs)\n\
             main = length (filter (\\x -> x > 3) [1 2 3 4 5])"
        ), 2);
    }

    #[test]
    fn closure_hof_apply_twice() {
        // (\f -> \x -> f (f x)) (\x -> x + 1) 0 = 2
        assert_eq!(jit(
            "apply_twice f x = f (f x)\n\
             main = apply_twice (\\x -> x + 1) 0"
        ), 2);
    }

    #[test]
    fn closure_capture() {
        // Closure captures a free variable
        assert_eq!(jit(
            "add_n n = \\x -> x + n\n\
             main = (add_n 10) 32"
        ), 42);
    }

    // ── String Tests ────────────────────────────────────

    fn jit_str(src: &str) -> String {
        compile_and_display(src)
            .unwrap_or_else(|e| panic!("Compile failed for:\n{}\nError: {}", src, e))
    }

    #[test]
    fn str_literal_display() {
        assert_eq!(jit_str("main = \"hello\""), "hello");
    }

    #[test]
    fn str_literal_length() {
        assert_eq!(jit("main = length \"hello\""), 5);
    }

    #[test]
    fn str_show_int() {
        assert_eq!(jit_str("main = show 42"), "42");
    }

    #[test]
    fn str_show_int_length() {
        assert_eq!(jit("main = length (show 42)"), 2);
    }

    #[test]
    fn str_show_large_int_length() {
        assert_eq!(jit("main = length (show 12345)"), 5);
    }

    #[test]
    fn str_concat_display() {
        assert_eq!(jit_str("main = \"foo\" ++ \"bar\""), "foobar");
    }

    #[test]
    fn str_concat_length() {
        assert_eq!(jit("main = length (\"foo\" ++ \"bar\")"), 6);
    }

    #[test]
    fn str_fizzbuzz_fizzbuzz() {
        let src = "fizzbuzz n = ? n % 15 == 0 -> \"FizzBuzz\" : ? n % 3 == 0 -> \"Fizz\" : ? n % 5 == 0 -> \"Buzz\" : show n";
        assert_eq!(jit_str(&format!("{}\nmain = fizzbuzz 15", src)), "FizzBuzz");
    }

    #[test]
    fn str_fizzbuzz_fizz() {
        let src = "fizzbuzz n = ? n % 15 == 0 -> \"FizzBuzz\" : ? n % 3 == 0 -> \"Fizz\" : ? n % 5 == 0 -> \"Buzz\" : show n";
        assert_eq!(jit_str(&format!("{}\nmain = fizzbuzz 3", src)), "Fizz");
    }

    #[test]
    fn str_fizzbuzz_buzz() {
        let src = "fizzbuzz n = ? n % 15 == 0 -> \"FizzBuzz\" : ? n % 3 == 0 -> \"Fizz\" : ? n % 5 == 0 -> \"Buzz\" : show n";
        assert_eq!(jit_str(&format!("{}\nmain = fizzbuzz 5", src)), "Buzz");
    }

    #[test]
    fn str_fizzbuzz_num() {
        let src = "fizzbuzz n = ? n % 15 == 0 -> \"FizzBuzz\" : ? n % 3 == 0 -> \"Fizz\" : ? n % 5 == 0 -> \"Buzz\" : show n";
        assert_eq!(jit_str(&format!("{}\nmain = fizzbuzz 7", src)), "7");
    }

    // ── List Comprehension Tests ────────────────────────────

    #[test]
    fn list_comp_basic() {
        // [x | x <- [1 2 3]] = [1 2 3], sum = 6
        assert_eq!(jit("main = sum [x | x <- [1 2 3]]"), 6);
    }

    #[test]
    fn list_comp_map() {
        // [x * 2 | x <- [1 2 3]] = [2 4 6], sum = 12
        assert_eq!(jit("main = sum [x * 2 | x <- [1 2 3]]"), 12);
    }

    #[test]
    fn list_comp_squares() {
        // [x * x | x <- [1 2 3 4 5]], sum = 55
        assert_eq!(jit("main = sum [x * x | x <- [1 2 3 4 5]]"), 55);
    }

    #[test]
    fn list_comp_with_guard() {
        // [x | x <- [1 2 3 4 5], x > 3] = [4 5], length = 2
        assert_eq!(jit("main = length [x | x <- [1 2 3 4 5], x > 3]"), 2);
    }

    #[test]
    fn list_comp_length() {
        assert_eq!(jit("main = length [x | x <- [1 2 3 4 5]]"), 5);
    }

    // ── Record Tests ────────────────────────────────────

    #[test]
    fn record_create_and_access() {
        assert_eq!(jit("p = {x = 3, y = 4}\nmain = p.x"), 3);
    }

    #[test]
    fn record_field_second() {
        assert_eq!(jit("p = {x = 3, y = 4}\nmain = p.y"), 4);
    }

    #[test]
    fn record_arithmetic() {
        assert_eq!(jit("p = {x = 3, y = 4}\nmain = p.x + p.y"), 7);
    }

    #[test]
    fn record_in_function() {
        assert_eq!(jit("dist_sq p = p.x * p.x + p.y * p.y\nmain = dist_sq {x = 3, y = 4}"), 25);
    }

    #[test]
    fn record_nested_int() {
        assert_eq!(jit("r = {a = 10, b = 20, c = 30}\nmain = r.a + r.b + r.c"), 60);
    }

    // ── String Equality Tests ───────────────────────────
    #[test]
    fn str_eq_equal() {
        assert_eq!(jit("main = ? \"hello\" == \"hello\" -> 1 : 0"), 1);
    }

    #[test]
    fn str_eq_not_equal() {
        assert_eq!(jit("main = ? \"hello\" == \"world\" -> 1 : 0"), 0);
    }

    #[test]
    fn str_eq_show_match() {
        // show 42 == "42" → true
        assert_eq!(jit("main = ? show 42 == \"42\" -> 1 : 0"), 1);
    }

    #[test]
    fn str_eq_show_no_match() {
        assert_eq!(jit("main = ? show 7 == \"42\" -> 1 : 0"), 0);
    }

    #[test]
    fn str_eq_int_still_works() {
        // Integer == must still work correctly
        assert_eq!(jit("main = ? 42 == 42 -> 1 : 0"), 1);
    }

    #[test]
    fn str_eq_int_false() {
        assert_eq!(jit("main = ? 42 == 7 -> 1 : 0"), 0);
    }

    // ── Record Pattern Tests ────────────────────────────

    #[test]
    fn record_pattern_var_binding() {
        // Pattern match on record: extract field via pattern
        assert_eq!(jit("get_x {x = v} = v\nmain = get_x {x = 42, y = 0}"), 42);
    }

    #[test]
    fn record_pattern_wildcard() {
        // Extract first field, ignore others
        assert_eq!(jit("first_field {a = v} = v\nmain = first_field {a = 7, b = 99}"), 7);
    }

    #[test]
    fn record_pattern_multi_field_get_x() {
        // Record pattern with two fields: one var, one wildcard
        let result = run_jit("
get_x {x = v, y = _} = v
main = get_x {x = 42, y = 99}
").unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn record_pattern_dist_sq() {
        // Record pattern binding both fields and using them in arithmetic
        let result = run_jit("
dist_sq {x = px, y = py} = px * px + py * py
main = dist_sq {x = 3, y = 4}
").unwrap();
        assert_eq!(result, 25);
    }

    #[test]
    fn record_pattern_three_fields() {
        // Record pattern with three fields, only bind first
        let result = run_jit("
get_first {x = v, y = _, z = _} = v
main = get_first {x = 7, y = 0, z = 0}
").unwrap();
        assert_eq!(result, 7);
    }

    #[test]
    fn str_neq_different() {
        assert_eq!(jit("main = ? \"hello\" != \"world\" -> 1 : 0"), 1);
    }

    #[test]
    fn str_neq_same() {
        assert_eq!(jit("main = ? \"hello\" != \"hello\" -> 1 : 0"), 0);
    }

    // ── Module Tests ────────────────────────────────────
    #[test]
    fn module_basic_function() {
        let src = "mod Math\n  square x = x * x\n\nuse Math (square)\n\nmain = square 7";
        assert_eq!(jit(src), 49);
    }

    #[test]
    fn module_constant() {
        let src = "mod Config\n  answer = 42\n\nuse Config (answer)\n\nmain = answer";
        assert_eq!(jit(src), 42);
    }

    #[test]
    fn module_multiple_imports() {
        let src = "mod Math\n  double x = x * 2\n  inc x = x + 1\n\nuse Math (double inc)\n\nmain = double (inc 20)";
        assert_eq!(jit(src), 42);
    }

    // ── ADT (Algebraic Data Types) in JIT ────────────────────

    fn run_jit(src: &str) -> Result<i64, String> {
        compile_and_run(src)
    }

    #[test]
    fn adt_maybe_none() {
        // 0-arity constructor: None matched via multi-equation
        let result = run_jit("
Maybe a = Just a | None
isNone (Just v) = 0
isNone None = 1
main = isNone None
").unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn adt_maybe_just_literal() {
        // Just wraps a value; multi-equation extracts it
        let result = run_jit("
Maybe a = Just a | None
fromJust (Just v) = v
fromJust None = 0
main = fromJust (Just 42)
").unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn adt_maybe_none_fallback() {
        // None branch returns fallback value
        let result = run_jit("
Maybe a = Just a | None
fromMaybe def (Just v) = v
fromMaybe def None = def
main = fromMaybe 99 None
").unwrap();
        assert_eq!(result, 99);
    }

    #[test]
    fn adt_shape_circle() {
        // Multi-variant ADT with fields, circle branch
        let result = run_jit("
Shape = Circle Int | Rect Int Int
area (Circle r) = r * r
area (Rect w h) = w * h
main = area (Circle 7)
").unwrap();
        assert_eq!(result, 49);
    }

    #[test]
    fn adt_shape_rect() {
        // Multi-variant ADT with fields, rect branch
        let result = run_jit("
Shape = Circle Int | Rect Int Int
area (Circle r) = r * r
area (Rect w h) = w * h
main = area (Rect 3 4)
").unwrap();
        assert_eq!(result, 12);
    }

    #[test]
    fn adt_recursive_list_sum() {
        // Recursive ADT: custom linked list summed via multi-equation
        let result = run_jit("
MyList a = MyCons Int (MyList a) | MyNil
mySum (MyCons h t) = h + mySum t
mySum MyNil = 0
main = mySum (MyCons 1 (MyCons 2 (MyCons 3 MyNil)))
").unwrap();
        assert_eq!(result, 6);
    }

    #[test]
    fn adt_nested_just_pair() {
        // Nested constructor pattern: Just (MkPair x y) → x + y
        let result = run_jit("
Maybe a = Just a | None
Pair a b = MkPair a b
unwrap (Just (MkPair x y)) = x + y
unwrap None = 0
main = unwrap (Just (MkPair 10 32))
").unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn adt_nested_wildcard_inner() {
        // Nested constructor with Wildcard sub-pat: Just (MkPair _ y) → y
        let result = run_jit("
Maybe a = Just a | None
Pair a b = MkPair a b
second (Just (MkPair _ y)) = y
second None = 0
main = second (Just (MkPair 99 7))
").unwrap();
        assert_eq!(result, 7);
    }

    #[test]
    fn adt_literal_sub_pattern_match() {
        // Literal sub-pattern: Just 0 = 100, Just n = n, None = -1
        let result = run_jit("
Maybe a = Just a | None
f (Just 0) = 100
f (Just n) = n
f None = -1
main = f (Just 0)
").unwrap();
        assert_eq!(result, 100);
    }

    #[test]
    fn adt_literal_sub_pattern_fallthrough() {
        // Literal sub-pattern fallthrough to variable case
        let result = run_jit("
Maybe a = Just a | None
f (Just 0) = 100
f (Just n) = n
f None = -1
main = f (Just 42)
").unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn adt_triple_nesting() {
        // Three levels of constructor nesting
        let result = run_jit("
Maybe a = Just a | None
triple (Just (Just (Just x))) = x
triple _ = 0
main = triple (Just (Just (Just 99)))
").unwrap();
        assert_eq!(result, 99);
    }

    #[test]
    fn adt_triple_nesting_fallback() {
        // Triple nesting fallback when inner is None
        let result = run_jit("
Maybe a = Just a | None
triple (Just (Just (Just x))) = x
triple _ = 0
main = triple (Just (Just None))
").unwrap();
        assert_eq!(result, 0);
    }

    // ── Phase 11.5: String literal patterns in JIT ────────

    #[test]
    fn str_pattern_match() {
        // String literal at top-level pattern, matching case
        let result = compile_and_display("
greet \"Alice\" = \"Hello, Alice\"
greet _ = \"Hi\"
main = greet \"Alice\"
").unwrap();
        assert_eq!(result, "Hello, Alice");
    }

    #[test]
    fn str_pattern_fallthrough() {
        // String literal pattern, fallthrough to wildcard
        let result = compile_and_display("
greet \"Alice\" = \"Hello, Alice\"
greet _ = \"Hi\"
main = greet \"Bob\"
").unwrap();
        assert_eq!(result, "Hi");
    }

    #[test]
    fn str_pattern_multi() {
        // Multiple string literal patterns
        let result = compile_and_display("
day \"Mon\" = \"Monday\"
day \"Tue\" = \"Tuesday\"
day _ = \"Other\"
main = day \"Tue\"
").unwrap();
        assert_eq!(result, "Tuesday");
    }

    #[test]
    fn str_pattern_in_constructor() {
        // String literal as sub-pattern inside a constructor
        let result = run_jit("
Maybe a = Just a | None
check (Just \"ok\") = 1
check _ = 0
main = check (Just \"ok\")
").unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn str_pattern_in_constructor_fallback() {
        // String sub-pattern mismatch falls back to wildcard
        let result = run_jit("
Maybe a = Just a | None
check (Just \"ok\") = 1
check _ = 0
main = check (Just \"fail\")
").unwrap();
        assert_eq!(result, 0);
    }

    // ── Float JIT Tests ──────────────────────────────────

    /// Helper: compile program, extract tagged float result as f64.
    fn run_float(src: &str) -> f64 {
        let v = compile_and_run(src)
            .unwrap_or_else(|e| panic!("Compile failed for:\n{}\nError: {}", src, e));
        runtime::decode_float(v)
            .unwrap_or_else(|| panic!("Expected float result, got i64={}", v))
    }

    #[test]
    fn float_literal() {
        let f = run_float("main = 3.14");
        assert!((f - 3.14).abs() < 1e-10, "Expected 3.14, got {}", f);
    }

    #[test]
    fn float_add() {
        let f = run_float("main = 1.5 + 2.5");
        assert!((f - 4.0).abs() < 1e-10, "Expected 4.0, got {}", f);
    }

    #[test]
    fn float_sub() {
        let f = run_float("main = 5.0 - 1.5");
        assert!((f - 3.5).abs() < 1e-10, "Expected 3.5, got {}", f);
    }

    #[test]
    fn float_mul() {
        let f = run_float("main = 2.0 * 3.0");
        assert!((f - 6.0).abs() < 1e-10, "Expected 6.0, got {}", f);
    }

    #[test]
    fn float_div() {
        let f = run_float("main = 7.0 / 2.0");
        assert!((f - 3.5).abs() < 1e-10, "Expected 3.5, got {}", f);
    }

    #[test]
    fn float_comparison_gt_true() {
        // 1.5 > 1.0 is true → return 1
        assert_eq!(jit("main = ? 1.5 > 1.0 -> 1 : 0"), 1);
    }

    #[test]
    fn float_comparison_gt_false() {
        // 0.5 > 1.0 is false → return 0
        assert_eq!(jit("main = ? 0.5 > 1.0 -> 1 : 0"), 0);
    }

    #[test]
    fn float_comparison_lt() {
        assert_eq!(jit("main = ? 1.0 < 2.0 -> 42 : 0"), 42);
    }

    #[test]
    fn float_cond_string_result() {
        // ? 1.5 > 1.0 -> "yes" : "no" → "yes"
        let result = compile_and_display("main = ? 1.5 > 1.0 -> \"yes\" : \"no\"").unwrap();
        assert_eq!(result, "yes");
    }

    #[test]
    fn float_in_function() {
        // half x = x / 2.0 ; main = half 9.0
        let f = run_float("half x = x / 2.0\nmain = half 9.0");
        assert!((f - 4.5).abs() < 1e-10, "Expected 4.5, got {}", f);
    }

    // ── Power operator tests ────────────────────────────

    #[test]
    fn jit_pow_int() {
        assert_eq!(jit("main = 2 ** 10"), 1024);
    }

    #[test]
    fn jit_pow_int_zero() {
        assert_eq!(jit("main = 5 ** 0"), 1);
    }

    #[test]
    fn jit_pow_int_one() {
        assert_eq!(jit("main = 7 ** 1"), 7);
    }

    #[test]
    fn jit_pow_float() {
        let f = run_float("main = 2.0 ** 3.0");
        assert!((f - 8.0).abs() < 1e-10, "Expected 8.0, got {}", f);
    }

    #[test]
    fn jit_pow_float_half() {
        // 4.0 ** 0.5 = 2.0 (square root)
        let f = run_float("main = 4.0 ** 0.5");
        assert!((f - 2.0).abs() < 1e-10, "Expected 2.0, got {}", f);
    }

    // ── Float math builtins ───────────────────────────────

    #[test]
    fn jit_sqrt() {
        let f = run_float("main = sqrt 4.0");
        assert!((f - 2.0).abs() < 1e-10, "Expected 2.0, got {}", f);
    }

    #[test]
    fn jit_sqrt_nine() {
        let f = run_float("main = sqrt 9.0");
        assert!((f - 3.0).abs() < 1e-10, "Expected 3.0, got {}", f);
    }

    #[test]
    fn jit_floor() {
        let f = run_float("main = floor 3.7");
        assert!((f - 3.0).abs() < 1e-10, "Expected 3.0, got {}", f);
    }

    #[test]
    fn jit_ceil() {
        let f = run_float("main = ceil 3.2");
        assert!((f - 4.0).abs() < 1e-10, "Expected 4.0, got {}", f);
    }

    #[test]
    fn jit_round_up() {
        let f = run_float("main = round 3.6");
        assert!((f - 4.0).abs() < 1e-10, "Expected 4.0, got {}", f);
    }

    #[test]
    fn jit_round_down() {
        let f = run_float("main = round 3.2");
        assert!((f - 3.0).abs() < 1e-10, "Expected 3.0, got {}", f);
    }

    #[test]
    fn jit_abs_int() {
        assert_eq!(jit("main = abs (0 - 5)"), 5);
    }

    #[test]
    fn jit_abs_pos_int() {
        assert_eq!(jit("main = abs 42"), 42);
    }
}
