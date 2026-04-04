# Tasks: Leaf Region Optimization

## Phase A: Static Analysis

- [x] A1. Add `fn needs_heap(expr: &CoreExpr) -> bool` in `compiler.rs`
  - Walk Core IR tree recursively
  - Return `true` for: MkList, MkClosure, Record, RecordUpdate, Con, Lit(Str), Scope, Spawn
  - Return `false` for: Var, Lit(Int|Bool), PrimOp, RuntimeError
  - Recurse into: App, Lam, Let, LetRec, Case (scrutinee + all alts), FieldAccess, Region
  - Conservative default: exhaustive match (no unknown nodes)

## Phase B: Conditional Region Emission

- [x] B1. Add `emit_regions: bool` field to `TcoContext` struct
- [x] B2. In `compile_function()`, call `needs_heap(inner)` and conditionally skip `region_enter` emission
- [x] B3. In TCO back-edge (~line 830), conditionally skip `region_exit` based on `tco_ctx.emit_regions`

## Phase C: Testing

- [x] C1. Run `cargo test` — all 960 tests pass (0 failures, 0 new warnings)
- [x] C2. fib(10) JIT returns 55 (covered by j5_jit_correctness_suite)
- [x] C3. Leak audit passes (m3_leak_audit_all_jit_programs includes fibonacci)
- [x] C4. Allocating functions (list comprehensions, strings) still emit regions — covered by existing JIT tests

---

## Dependency Chain

A1 → B1 → B2 → B3 → C1 → C2 → C3 → C4

**Critical path**: A → B → C (implement analysis, wire in, test)
