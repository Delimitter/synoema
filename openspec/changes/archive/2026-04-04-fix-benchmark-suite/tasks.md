# Tasks: Fix Benchmark Suite

## Benchmark .sno rewrites
- [x] Rewrite `binary_search.sno` — guard-based `nth`, hoisted `mid`/`val` bindings before ternary
- [x] Rewrite `mergesort.sno` — guard-based `merge`/`take_n`/`drop_n`, no multi-arg numeric patterns
- [x] Rewrite `matrix_mult.sno` — `get`/`dot`/`col`/`row_mul` helpers with where-bindings
- [x] Rewrite `string_ops.sno` — replaced `str_length` with `str_len` (correct builtin name)
- [x] Rewrite `tree_traverse.sno` — guard-based `join_sp`, no `[x]` singleton pattern

## Runner fixes
- [x] Add `-std=c++17` to g++ args in `runtime.rs`
- [x] Add explicit FAIL log line when benchmark execution fails

## Validation
- [x] All 5 rewritten .sno files exit 0 in interpreter (`run`) mode with correct output
- [x] All 5 rewritten .sno files exit 0 in JIT mode
- [x] C++ tree_traverse compiles successfully with `-std=c++17`
- [x] `cargo test` passes (875 tests, 0 failures)
- [x] `cargo build` for runner passes
- [x] All existing benchmarks (collatz, factorial, fibonacci, filter_map, fizzbuzz, gcd, quicksort) still pass

## Known limitations (out of scope)
- JIT list display returns raw tagged pointers for mergesort/matrix_mult (known JIT issue)
- Token counts increased for rewritten files due to guard-based workarounds for parser limitations
