# Optimize: Skip Region Enter/Exit for Non-Allocating JIT Functions

## Problem

Every user-defined function compiled by the JIT unconditionally emits `synoema_region_enter()` at entry and `synoema_region_exit()` before TCO back-edges (compiler.rs:524-527). These are FFI calls into the Rust runtime that push/pop the arena region stack.

For functions that never allocate heap memory (no lists, strings, closures, records), these calls are pure overhead. Fibonacci `fib(30)` makes 2,178,309 function calls — each paying the region FFI cost for zero benefit.

## Proposed Solution

Add a static analysis pass (`needs_heap`) that walks the Core IR body of a function before compilation. If the body contains no heap-allocating nodes (MkList, MkClosure, Record, Con, string literals, list comprehensions), skip emitting region_enter/region_exit entirely.

## Scope

- `lang/crates/synoema-codegen/src/compiler.rs` — conditional region emission in `compile_function`
- No changes to runtime, parser, evaluator, or Core IR

## Expected Impact

- ~15-25% speedup on recursive numeric functions (fib, factorial, ackermann)
- Zero impact on correctness — non-allocating functions never use the region stack
- No behavioral change for allocating functions

## Risk

Low. The analysis is conservative: if uncertain, emit regions (safe default).
