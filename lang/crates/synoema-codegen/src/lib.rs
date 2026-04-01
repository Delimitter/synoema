//! # synoema-codegen
//! Cranelift-based native code generator for the Synoema programming language.
//!
//! Compiles Core IR to native machine code via JIT.

pub mod compiler;
pub mod runtime;
pub use compiler::{Compiler, CompileError};

/// Parse, desugar, and JIT-compile an Synoema program, returning main() result.
pub fn compile_and_run(source: &str) -> Result<i64, String> {
    let program = synoema_parser::parse(source)
        .map_err(|e| format!("Parse error: {}", e))?;
    let core = synoema_core::desugar_program(&program);
    let mut compiler = Compiler::new()
        .map_err(|e| format!("{}", e))?;
    compiler.compile_and_run(&core)
        .map_err(|e| format!("{}", e))
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
}
