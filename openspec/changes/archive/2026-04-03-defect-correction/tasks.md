# Tasks: Defect Correction

## Checklist

- [x] T1: Fix row polymorphism unification when r1 == r2 (unify.rs)
- [x] T2: Add test — same row var, conflicting field types → error (3 tests)
- [x] T3: Add `RuntimeError(String)` to `CoreExpr` (core_ir.rs)
- [x] T4: Emit RuntimeError in desugar.rs fallback + wrap_lambdas + multi-eq Case
- [x] T5: Handle RuntimeError in interpreter — N/A (uses surface AST)
- [x] T6: Handle RuntimeError in JIT (compiler.rs + runtime.rs via FFI)
- [x] T7: Add test — non-exhaustive match → RuntimeError in desugared output
- [x] T8: Fix constructor pattern type checking in infer_pattern (infer.rs)
- [x] T9: Add test — wrong arity constructor pattern → type error
- [x] T10: Verify all tests pass (707), 0 warnings
