# Tasks: Fix HM Type Checker Performance

- [x] Add `cached_ftv` field to `TypeEnv` in types.rs
- [x] Update `TypeEnv::ftv()` to use/populate cache
- [x] Update `TypeEnv::insert()` to invalidate cache
- [x] Remove `env = env.apply(&final_subst)` from main inference loop in infer.rs (both impl and func passes)
- [x] Keep apply on the inferred type itself (not the env)
- [x] `cargo test -p synoema-types` — 90 passed, 0 failures
- [x] `cargo test -p synoema-eval` — 238 passed, 0 failures
- [x] `cargo test -p synoema-codegen` — 217 passed, 0 failures
- [x] Perf test: Map prelude (15 functions) type-checks in 0.09s (was 2+ min)
- [x] Removed `map_fold_list` from prelude (JIT verifier error — separate issue)
