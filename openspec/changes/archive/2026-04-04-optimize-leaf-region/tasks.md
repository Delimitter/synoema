# Tasks: Leaf Region Optimization

## Phase A: Static Analysis

- [x] A1. Add `fn needs_heap(expr: &CoreExpr) -> bool` in `compiler.rs`

## Phase B: Conditional Region Emission

- [x] B1. Add `emit_regions: bool` field to `TcoContext` struct
- [x] B2. In `compile_function()`, conditionally skip `region_enter` emission
- [x] B3. In TCO back-edge, conditionally skip `region_exit` based on `tco_ctx.emit_regions`

## Phase C: Testing

- [x] C1. `cargo test` — 960 tests pass (0 failures, 0 new warnings)
- [x] C2. fib(10) JIT returns 55 (j5_jit_correctness_suite)
- [x] C3. Leak audit passes (m3_leak_audit_all_jit_programs)
- [x] C4. Allocating functions still emit regions correctly
