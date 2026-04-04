# Proposal: Fix Benchmark Suite

## Problem
5 of 12 runtime benchmarks silently fail for Synoema (exit code 1, runner skips them). C++ tree_traverse fails to compile (missing -std=c++17). Runner doesn't report failures explicitly — silent skip skews averages.

## Root Causes
1. **binary_search.sno** — where-bindings inside ternary branches (unsupported syntax)
2. **mergesort.sno** — cons `:` without parens in ternary then-branch + `take`/`drop` builtins missing
3. **matrix_mult.sno** — uses `foldl`/`zip` (not available in run/jit mode) + `[x]` singleton pattern
4. **string_ops.sno** — `str_length` builtin doesn't exist
5. **tree_traverse.sno** — `[x]` singleton list pattern unsupported
6. **C++ runner** — `g++` invoked without `-std=c++17`, `std::variant` requires it on macOS
7. **Runner** — `measure_once` returns `None` on failure, loop silently `continue`s

## Scope
- Rewrite 5 .sno files using supported syntax (guards, hoisted bindings, inline helpers)
- Add `-std=c++17` to C++ compilation in runner
- Add explicit FAIL logging when benchmark fails
- Do NOT change language compiler/parser/JIT — fix benchmarks to work within current capabilities

## Constraints
- Available builtins (file mode): `length`, `head`, `tail`, `map`, `filter`, `show`, `++`
- NOT available (file mode): `foldl`, `zip`, `index`, `take`, `drop`, `str_length`, `reverse`
- Multi-arg numeric literal patterns are buggy (e.g., `f 0 xs` matches all — use guards)
- Where-bindings inside ternary branches not supported (hoist before ternary)
- Cons in ternary then-branch requires parens: `? cond -> (x : rest)`
- `[x]` singleton pattern unsupported (use `length xs == 1` guard)
- JIT displays raw tagged pointers for list results (known, separate issue — not in scope)
