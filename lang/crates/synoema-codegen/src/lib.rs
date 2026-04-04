// SPDX-License-Identifier: BUSL-1.1
// Copyright (c) 2025-present Andrey Bubnov

//! # synoema-codegen
//! Cranelift-based native code generator for the Synoema programming language.
//!
//! Compiles Core IR to native machine code via JIT.

pub mod compiler;
pub mod runtime;
pub use compiler::{Compiler, CompileError};
pub use runtime::{arena_reset, arena_offset, arena_overflow_count, arena_region_depth};

use std::path::Path;
use synoema_diagnostic::{Diagnostic, codes};
use synoema_parser::{ImportError, ImportErrorCode};

fn parse_err(e: &synoema_parser::ParseError) -> Diagnostic {
    Diagnostic::error(codes::PARSE_UNEXPECTED_TOKEN, e.message.clone())
        .with_span(e.span)
}

fn import_err(e: ImportError) -> Diagnostic {
    let code = match e.code {
        ImportErrorCode::Cycle => codes::IMPORT_CYCLE,
        ImportErrorCode::NotFound => codes::IMPORT_NOT_FOUND,
        ImportErrorCode::ParseError => codes::PARSE_UNEXPECTED_TOKEN,
    };
    Diagnostic::error(code, e.message).with_span(e.span)
}

fn compile_err(e: CompileError) -> Diagnostic {
    Diagnostic::error(codes::COMPILE_ERROR, format!("{}", e))
}

// ── Prelude ─────────────────────────────────────────────

const PRELUDE: &str = include_str!("../../../prelude/prelude.sno");

fn prepend_prelude(user_source: &str) -> String {
    format!("{}\n{}", PRELUDE, user_source)
}

/// Parse, desugar, and JIT-compile a Synoema program, returning main() result as i64.
/// Strings are returned as tagged i64 pointers (use `display_result` for human-readable output).
pub fn compile_and_run(source: &str) -> Result<i64, Diagnostic> {
    compile_and_run_with_base_dir(source, None)
}

/// Like `compile_and_run` but with import resolution from `base_dir`.
pub fn compile_and_run_with_base_dir(source: &str, base_dir: Option<&Path>) -> Result<i64, Diagnostic> {
    let full_source = prepend_prelude(source);
    let program = synoema_parser::parse(&full_source)
        .map_err(|e| parse_err(&e))?;
    let program = if let Some(dir) = base_dir {
        synoema_parser::resolve_imports(program, dir).map_err(import_err)?
    } else { program };
    let program = synoema_types::resolve_modules(program);
    let core = synoema_core::desugar_program(&program);
    let core = synoema_core::optimize_program(core);
    let core = synoema_core::annotate_regions(core);
    let mut compiler = Compiler::new()
        .map_err(compile_err)?;
    let result = compiler.compile_and_run(&core)
        .map_err(compile_err);
    crate::runtime::arena_reset(); // Free all heap allocations from this run
    result
}

/// Parse, desugar, JIT-compile and return main() result as a display string.
/// Handles both integer results and tagged string results.
pub fn compile_and_display(source: &str) -> Result<String, Diagnostic> {
    compile_and_display_with_base_dir(source, None)
}

/// Like `compile_and_display` but with import resolution from `base_dir`.
pub fn compile_and_display_with_base_dir(source: &str, base_dir: Option<&Path>) -> Result<String, Diagnostic> {
    let result = compile_and_run_with_base_dir(source, base_dir)?;
    Ok(runtime::display_value(result))
}

/// Extract Core IR for build artifacts without executing.
pub fn extract_core_ir(source: &str, base_dir: Option<&Path>) -> Result<String, Diagnostic> {
    let program = synoema_parser::parse(source)
        .map_err(|e| parse_err(&e))?;
    let program = if let Some(dir) = base_dir {
        synoema_parser::resolve_imports(program, dir).map_err(import_err)?
    } else { program };
    let program = synoema_types::resolve_modules(program);
    let core = synoema_core::desugar_program(&program);
    let core = synoema_core::optimize_program(core);
    let core = synoema_core::annotate_regions(core);
    Ok(format!("{:#?}", core))
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
        compile_and_run(src).map_err(|e| e.to_string())
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
my_unwrap (Just (MkPair x y)) = x + y
my_unwrap None = 0
main = my_unwrap (Just (MkPair 10 32))
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

    // ── Phase 14b: IO / Effects in JIT ────────────────────

    #[test]
    fn jit_unit_literal() {
        // () compiles to 0 (unit)
        assert_eq!(jit("main = ()"), 0);
    }

    #[test]
    fn jit_print_int_returns_unit() {
        // print returns 0 (unit)
        assert_eq!(jit("main = print 42"), 0);
    }

    #[test]
    fn jit_print_string_returns_unit() {
        assert_eq!(jit(r#"main = print "hello""#), 0);
    }

    #[test]
    fn jit_seq_returns_right() {
        // a ; b returns b's value
        assert_eq!(jit("main = () ; 99"), 99);
    }

    #[test]
    fn jit_seq_two_prints() {
        // print "hello" ; print "world" — both execute, returns 0
        assert_eq!(jit(r#"main = print "hello" ; print "world""#), 0);
    }

    #[test]
    fn jit_seq_print_then_int() {
        assert_eq!(jit(r#"main = print "hi" ; 42"#), 42);
    }

    #[test]
    fn jit_print_bool() {
        assert_eq!(jit("main = print true"), 0);
    }

    #[test]
    fn jit_seq_chain() {
        assert_eq!(jit(r#"main = print "a" ; print "b" ; print "c""#), 0);
    }

    // ── Phase 15a: show (any type), list ==, range ─────────

    #[test]
    fn jit_show_float() {
        assert_eq!(jit_str("main = show 3.14"), "3.14");
    }

    #[test]
    fn jit_show_float_whole() {
        assert_eq!(jit_str("main = show 2.0"), "2.0");
    }

    #[test]
    fn jit_show_str_identity() {
        // show on a string should return the string unchanged
        assert_eq!(jit_str(r#"main = show "hello""#), "hello");
    }

    #[test]
    fn jit_show_int_still_works() {
        assert_eq!(jit_str("main = show 42"), "42");
    }

    #[test]
    fn jit_list_eq_equal() {
        assert_eq!(jit("main = ? [1 2 3] == [1 2 3] -> 1 : 0"), 1);
    }

    #[test]
    fn jit_list_eq_not_equal() {
        assert_eq!(jit("main = ? [1 2 3] == [1 2 4] -> 1 : 0"), 0);
    }

    #[test]
    fn jit_list_eq_different_length() {
        assert_eq!(jit("main = ? [1 2] == [1 2 3] -> 1 : 0"), 0);
    }

    #[test]
    fn jit_list_eq_empty() {
        assert_eq!(jit("main = ? [] == [] -> 1 : 0"), 1);
    }

    #[test]
    fn jit_list_eq_int_small() {
        // Tests that integers 2, 6 etc. are not falsely treated as strings
        assert_eq!(jit("main = ? [2 4 6] == [2 4 6] -> 1 : 0"), 1);
    }

    #[test]
    fn jit_range_length() {
        assert_eq!(jit("main = length [1..10]"), 10);
    }

    #[test]
    fn jit_range_sum() {
        assert_eq!(jit("main = sum [1..10]"), 55);
    }

    #[test]
    fn jit_range_single() {
        assert_eq!(jit("main = sum [5..5]"), 5);
    }

    #[test]
    fn jit_range_in_comp() {
        // List comprehension over range
        assert_eq!(jit("main = sum [x * 2 | x <- [1..5]]"), 30);
    }

    // ── Phase 15b: show Bool, show List ────────────────────

    #[test]
    fn jit_show_bool_true() {
        assert_eq!(jit_str("main = show true"), "true");
    }

    #[test]
    fn jit_show_bool_false() {
        assert_eq!(jit_str("main = show false"), "false");
    }

    #[test]
    fn jit_show_list_ints() {
        assert_eq!(jit_str("main = show [1 2 3]"), "[1 2 3]");
    }

    #[test]
    fn jit_show_list_cons() {
        assert_eq!(jit_str("main = show (1 : 2 : 3 : [])"), "[1 2 3]");
    }

    #[test]
    fn jit_show_list_floats() {
        assert_eq!(jit_str("main = show [1.5 2.5]"), "[1.5 2.5]");
    }

    #[test]
    fn jit_show_list_single() {
        assert_eq!(jit_str("main = show [42]"), "[42]");
    }

    #[test]
    fn jit_show_in_concat() {
        // show can be used in string concatenation
        assert_eq!(jit_str(r#"main = "len=" ++ show (length [1 2 3])"#), "len=3");
    }

    // ── Phase 15c: show for ADTs and Records ────────────────

    #[test]
    fn jit_show_adt_none() {
        let src = "Maybe a = Just a | None\nmain = show None";
        assert_eq!(jit_str(src), "None");
    }

    #[test]
    fn jit_show_adt_just_int() {
        let src = "Maybe a = Just a | None\nmain = show (Just 42)";
        assert_eq!(jit_str(src), "Just 42");
    }

    #[test]
    fn jit_show_adt_two_fields() {
        let src = "Shape = Rect Int Int\nmain = show (Rect 3 4)";
        assert_eq!(jit_str(src), "Rect 3 4");
    }

    #[test]
    fn jit_show_adt_nested() {
        // Nested constructor: Just (Just 7) → "Just (Just 7)"
        let src = "Maybe a = Just a | None\nmain = show (Just (Just 7))";
        assert_eq!(jit_str(src), "Just (Just 7)");
    }

    #[test]
    fn jit_show_record_two_fields() {
        assert_eq!(jit_str("main = show {x = 3, y = 4}"), "{x = 3, y = 4}");
    }

    #[test]
    fn jit_show_record_single_field() {
        assert_eq!(jit_str("main = show {n = 42}"), "{n = 42}");
    }

    #[test]
    fn jit_show_adt_in_concat() {
        let src = "Maybe a = Just a | None\nmain = \"val=\" ++ show (Just 99)";
        assert_eq!(jit_str(src), "val=Just 99");
    }

    #[test]
    fn jit_show_record_float_field() {
        assert_eq!(jit_str("main = show {pi = 3.14}"), "{pi = 3.14}");
    }

    // ── Logical operators in JIT ────────────────────────────────

    #[test]
    fn jit_and_true_true() { assert_eq!(jit("main = ? true && true -> 1 : 0"), 1); }

    #[test]
    fn jit_and_true_false() { assert_eq!(jit("main = ? true && false -> 1 : 0"), 0); }

    #[test]
    fn jit_or_false_true() { assert_eq!(jit("main = ? false || true -> 1 : 0"), 1); }

    #[test]
    fn jit_or_false_false() { assert_eq!(jit("main = ? false || false -> 1 : 0"), 0); }

    // ── Float equality and comparison in JIT ─────────────────────

    #[test]
    fn jit_float_eq_true() { assert_eq!(jit("main = ? 1.5 == 1.5 -> 1 : 0"), 1); }

    #[test]
    fn jit_float_eq_false() { assert_eq!(jit("main = ? 1.5 == 2.0 -> 1 : 0"), 0); }

    #[test]
    fn jit_float_neq_true() { assert_eq!(jit("main = ? 1.5 != 2.0 -> 1 : 0"), 1); }

    #[test]
    fn jit_float_neq_false() { assert_eq!(jit("main = ? 1.5 != 1.5 -> 1 : 0"), 0); }

    #[test]
    fn jit_float_lte_equal() { assert_eq!(jit("main = ? 1.5 <= 1.5 -> 1 : 0"), 1); }

    #[test]
    fn jit_float_lte_less() { assert_eq!(jit("main = ? 1.0 <= 2.0 -> 1 : 0"), 1); }

    #[test]
    fn jit_float_lte_false() { assert_eq!(jit("main = ? 2.0 <= 1.0 -> 1 : 0"), 0); }

    #[test]
    fn jit_float_gte_equal() { assert_eq!(jit("main = ? 2.5 >= 2.5 -> 1 : 0"), 1); }

    #[test]
    fn jit_float_gte_greater() { assert_eq!(jit("main = ? 3.0 >= 1.5 -> 1 : 0"), 1); }

    // ── GCD and Quicksort in JIT ─────────────────────────────────

    #[test]
    fn jit_gcd() {
        assert_eq!(jit("gcd a 0 = a\ngcd a b = gcd b (a % b)\nmain = gcd 48 18"), 6);
    }

    #[test]
    fn jit_quicksort_sum() {
        let src = "\
qsort [] = []
qsort (p:xs) = qsort lo ++ [p] ++ qsort hi
  lo = [x | x <- xs , x <= p]
  hi = [x | x <- xs , x > p]
main = sum (qsort [3 1 4 1 5 9 2 6])";
        assert_eq!(jit(src), 31);
    }

    // ── Range + filter in comprehensions ─────────────────────────

    #[test]
    fn jit_range_filter_evens() {
        assert_eq!(jit("main = length [x | x <- [1..10], x % 2 == 0]"), 5);
    }

    #[test]
    fn jit_range_filter_sum_evens() {
        assert_eq!(jit("main = sum [x | x <- [1..10], x % 2 == 0]"), 30);
    }

    // ── Pipe operator in JIT ──────────────────────────────────────

    #[test]
    fn jit_pipe_simple() {
        assert_eq!(jit("double x = x * 2\nmain = 21 |> double"), 42);
    }

    #[test]
    fn jit_pipe_chain() {
        assert_eq!(jit("double x = x * 2\nmain = 5 |> double |> double"), 20);
    }

    // ── Record with float field ───────────────────────────────────

    #[test]
    fn jit_record_float_field() {
        let f = run_float("p = {x = 3.0, y = 4.0}\nmain = p.x");
        assert!((f - 3.0).abs() < 1e-10, "Expected 3.0, got {}", f);
    }

    // ── Power right associativity ─────────────────────────────────

    #[test]
    fn jit_pow_right_assoc() {
        // 2 ** 3 ** 2 = 2 ** (3 ** 2) = 2 ** 9 = 512
        assert_eq!(jit("main = 2 ** 3 ** 2"), 512);
    }

    // ── Phase 16: Type class Show in JIT ──────────────────────────

    #[test]
    fn jit_typeclass_show_red() {
        let src = "\
Color = Red | Green | Blue
impl Show Color
  show Red = \"Red\"
  show Green = \"Green\"
  show Blue = \"Blue\"
main = show Red";
        assert_eq!(jit_str(src), "Red");
    }

    #[test]
    fn jit_typeclass_show_green() {
        let src = "\
Color = Red | Green | Blue
impl Show Color
  show Red = \"Red\"
  show Green = \"Green\"
  show Blue = \"Blue\"
main = show Green";
        assert_eq!(jit_str(src), "Green");
    }

    #[test]
    fn jit_typeclass_show_with_trait_decl() {
        let src = "\
Color = Red | Green | Blue
trait Show a
  show : a -> String
impl Show Color
  show Red = \"Red\"
  show Green = \"Green\"
  show Blue = \"Blue\"
main = show Blue";
        assert_eq!(jit_str(src), "Blue");
    }

    #[test]
    fn jit_typeclass_show_in_concat() {
        let src = "\
Color = Red | Green | Blue
impl Show Color
  show Red = \"Red\"
  show Green = \"Green\"
  show Blue = \"Blue\"
main = \"color=\" ++ show Green";
        assert_eq!(jit_str(src), "color=Green");
    }

    #[test]
    fn jit_show_bool_var_true() {
        assert_eq!(jit_str("main = show_bool true"), "true");
    }

    #[test]
    fn jit_show_bool_var_false() {
        assert_eq!(jit_str("main = show_bool false"), "false");
    }

    #[test]
    fn jit_show_bool_comparison() {
        // show on a comparison → is_bool_expr detects it, calls synoema_show_bool
        assert_eq!(jit_str("main = show (3 > 2)"), "true");
    }

    #[test]
    fn jit_show_bool_eq_false() {
        assert_eq!(jit_str("main = show (1 == 2)"), "false");
    }

    // ── Phase 17: map / filter / foldl in JIT ─────────────────────

    #[test]
    fn jit_map_double() {
        assert_eq!(jit("main = sum (map (\\x -> x * 2) [1 2 3])"), 12);
    }

    #[test]
    fn jit_map_increment() {
        assert_eq!(jit("main = sum (map (\\x -> x + 1) [1 2 3 4 5])"), 20);
    }

    #[test]
    fn jit_map_squares_range() {
        assert_eq!(jit("main = sum (map (\\x -> x * x) [1..5])"), 55);
    }

    #[test]
    fn jit_filter_even() {
        assert_eq!(jit("main = sum (filter (\\x -> x % 2 == 0) [1 2 3 4 5 6])"), 12);
    }

    #[test]
    fn jit_filter_gt() {
        assert_eq!(jit("main = length (filter (\\x -> x > 3) [1 2 3 4 5])"), 2);
    }

    #[test]
    fn jit_filter_range() {
        assert_eq!(jit("main = length (filter (\\x -> x % 2 == 0) [1..100])"), 50);
    }

    #[test]
    fn jit_foldl_sum() {
        assert_eq!(jit("main = foldl (\\acc x -> acc + x) 0 [1 2 3 4 5]"), 15);
    }

    #[test]
    fn jit_foldl_product() {
        assert_eq!(jit("main = foldl (\\acc x -> acc * x) 1 [1 2 3 4 5]"), 120);
    }

    // ── TCO (tail-call optimization) tests ──────────────────────────────

    #[test]
    fn jit_tco_countdown() {
        // Simple tail-recursive countdown — would overflow without TCO
        assert_eq!(jit(
            "countdown n = ? n == 0 -> 0 : countdown (n - 1)\nmain = countdown 1000000"
        ), 0);
    }

    #[test]
    fn jit_tco_sum_acc() {
        // Accumulator-style tail recursion: sum 1..10
        assert_eq!(jit(
            "sum_to n acc = ? n == 0 -> acc : sum_to (n - 1) (acc + n)\nmain = sum_to 10 0"
        ), 55);
    }

    #[test]
    fn jit_tco_gcd_unchanged() {
        // gcd is tail-recursive — result should be unchanged
        assert_eq!(jit(
            "gcd a b = ? b == 0 -> a : gcd b (a % b)\nmain = gcd 1071 462"
        ), 21);
    }

    #[test]
    fn jit_tco_factorial_unchanged() {
        // factorial is NOT tail-recursive (n * fact(n-1)) — result should still be correct
        assert_eq!(jit(
            "fact 0 = 1\nfact n = n * fact (n - 1)\nmain = fact 10"
        ), 3628800);
    }

    #[test]
    fn jit_tco_deep_acc() {
        // Deep tail recursion with accumulator — 1M iterations
        assert_eq!(jit(
            "sum_to n acc = ? n == 0 -> acc : sum_to (n - 1) (acc + n)\nmain = sum_to 1000000 0"
        ), 500000500000);
    }

    // ── String Stdlib in JIT ─────────────────────────────

    #[test]
    fn jit_str_slice_basic() {
        assert_eq!(jit_str("main = str_slice \"hello world\" 0 5"), "hello");
    }

    #[test]
    fn jit_str_slice_middle() {
        assert_eq!(jit_str("main = str_slice \"hello world\" 6 11"), "world");
    }

    #[test]
    fn jit_str_slice_clamped() {
        assert_eq!(jit_str("main = str_slice \"hi\" 0 100"), "hi");
    }

    #[test]
    fn jit_str_find_found() {
        assert_eq!(jit("main = str_find \"hello world\" \" \" 0"), 5);
    }

    #[test]
    fn jit_str_find_not_found() {
        assert_eq!(jit("main = str_find \"hello\" \"xyz\" 0"), -1);
    }

    #[test]
    fn jit_str_find_from_offset() {
        assert_eq!(jit("main = str_find \"abcabc\" \"bc\" 2"), 4);
    }

    #[test]
    fn jit_str_starts_with_true() {
        assert_eq!(jit("main = str_starts_with \"hello\" \"hel\""), 1);
    }

    #[test]
    fn jit_str_starts_with_false() {
        assert_eq!(jit("main = str_starts_with \"hello\" \"world\""), 0);
    }

    #[test]
    fn jit_str_trim_spaces() {
        assert_eq!(jit_str("main = str_trim \"  hello  \""), "hello");
    }

    #[test]
    fn jit_str_len_basic() {
        assert_eq!(jit("main = str_len \"hello\""), 5);
    }

    #[test]
    fn jit_str_len_empty() {
        assert_eq!(jit("main = str_len \"\""), 0);
    }

    #[test]
    fn jit_json_escape_quotes() {
        assert_eq!(jit_str("main = json_escape \"he said \\\"hi\\\"\""), "he said \\\"hi\\\"");
    }

    #[test]
    fn jit_json_escape_backslash() {
        assert_eq!(jit_str("main = json_escape \"a\\\\b\""), "a\\\\b");
    }

    // ── Region inference tests ────────────────────────

    #[test]
    fn jit_region_tco_loop_basic() {
        // Tail-recursive countdown — TCO auto-region should free per-iteration heap
        assert_eq!(jit("countdown n = ? n == 0 -> 0 : countdown (n - 1)\nmain = countdown 100000"), 0);
    }

    #[test]
    fn jit_region_tco_sum_acc() {
        // Tail-recursive sum with accumulator
        assert_eq!(jit("sum_to n acc = ? n == 0 -> acc : sum_to (n - 1) (acc + n)\nmain = sum_to 1000 0"), 500500);
    }

    #[test]
    fn jit_region_non_escaping_let() {
        // Non-escaping list: result is length (int), list can be freed
        assert_eq!(jit("main = length [1 2 3 4 5]"), 5);
    }

    #[test]
    fn jit_region_escaping_let() {
        // Escaping list: result IS the list, must survive
        assert_eq!(jit("main = head [42 1 2]"), 42);
    }

    #[test]
    fn jit_region_sum_range() {
        // sum of [1..100] — range allocates, sum consumes
        assert_eq!(jit("main = sum [1..100]"), 5050);
    }

    // ── Prelude: Result + error in JIT ───────────────

    #[test]
    fn jit_result_unwrap_ok() {
        assert_eq!(jit("main = unwrap (Ok 42)"), 42);
    }

    #[test]
    fn jit_result_unwrap_or_on_err() {
        assert_eq!(jit("main = unwrap_or 0 (Err \"fail\")"), 0);
    }

    #[test]
    fn jit_result_is_ok() {
        assert_eq!(jit("main = ? is_ok (Ok 1) -> 1 : 0"), 1);
    }

    // ── display_value: list/ADT/record formatting ────────────

    #[test]
    fn display_list_literal() {
        assert_eq!(jit_str("main = [1 2 3]"), "[1 2 3]");
    }

    #[test]
    fn display_list_tail() {
        assert_eq!(jit_str("main = tail [1 2 3]"), "[2 3]");
    }

    #[test]
    fn display_list_empty_is_zero() {
        assert_eq!(jit_str("main = tail [1]"), "0");
    }

    #[test]
    fn display_list_singleton() {
        assert_eq!(jit_str("main = [42]"), "[42]");
    }

    #[test]
    fn display_list_cons() {
        assert_eq!(jit_str("main = 1 : 2 : 3 : []"), "[1 2 3]");
    }

    #[test]
    fn display_list_range() {
        assert_eq!(jit_str("main = [1..5]"), "[1 2 3 4 5]");
    }

    #[test]
    fn display_adt_none() {
        assert_eq!(jit_str("Maybe a = Just a | None\nmain = None"), "None");
    }

    #[test]
    fn display_adt_just() {
        assert_eq!(jit_str("Maybe a = Just a | None\nmain = Just 42"), "Just 42");
    }
}
