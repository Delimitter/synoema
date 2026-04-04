# Optimize: Tree Recursion → Linear Accumulator Transformation

## Problem

Binary tree-recursive functions like Fibonacci have O(2^n) call complexity:
- `fib 30` makes 2,178,309 function calls
- `fib 40` makes 331,160,281 calls
- Each call has overhead even with the leaf-region optimization

The algorithmic complexity is the root cause — no per-call optimization can fix exponential growth.

## Proposed Solution

Add a Core IR optimization pass that detects the pattern:
```
f base0 = val0
f base1 = val1
f n = f(n-step1) OP f(n-step2)
```

And transforms it into an accumulator-based linear iteration:
```
f n = f__worker n val0 val1
f__worker base0 a b = a
f__worker n a b = f__worker (n-1) b (a OP b)
```

The worker function is tail-recursive, so TCO (already implemented) converts it to a loop.

## Scope

- `lang/crates/synoema-core/src/optimize.rs` — new `linearize_tree_recursion` pass
- Program-level transformation (operates on `CoreProgram.defs`)
- No changes to parser, JIT, runtime, or evaluator

## Expected Impact

- fib(30): O(2^30) → O(30) calls — ~100,000x fewer function calls
- fib(40): O(2^40) → O(40) — from ~5 seconds to microseconds
- Combined with leaf-region optimization: near-native performance for numeric recursion

## Risk

Medium. Pattern detection must be precise to avoid incorrect transformations. Conservative: only fires for exact pattern match.
