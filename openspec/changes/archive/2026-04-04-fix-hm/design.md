# Design: Fix HM Type Checker Performance

## Fix 1: Skip env.apply for generalized schemes (PRIMARY)

In `infer.rs` inference loop (line ~115):
```rust
env = env.apply(&final_subst);  // BEFORE: O(N) walk of entire env
```

After generalizing function A, its scheme `∀a b. a → b → a` has all free vars quantified.
A later substitution `[t7 → Int]` can't affect it because `a` and `b` are bound.
Only the CURRENT function's self-recursive binding and any un-generalized entries need apply.

**Change:** Replace `env = env.apply(&final_subst)` with targeted apply — only apply to entries that have free type variables matching the substitution domain.

Simplest correct approach: maintain an `accumulated_subst` and apply lazily during `instantiate()` instead of eagerly after each function.

## Fix 2: Cache ftv on TypeEnv (SECONDARY)

`TypeEnv::ftv()` walks all entries. Called once per function in `generalize`.

**Change:** Add `cached_ftv: HashSet<TyVarId>` to TypeEnv.
- On insert: union with new scheme's ftv
- On remove: recompute (rare, only for self-recursive binding)
- On apply: invalidate cache (but with Fix 1, apply is rare)

## Implementation

### File: `types.rs`

Add `cached_ftv` field to `TypeEnv`:
```rust
pub struct TypeEnv {
    bindings: HashMap<String, Scheme>,
    cached_ftv: Option<HashSet<TyVarId>>,
}
```

Methods to maintain cache:
- `insert()` → invalidate cache (set to None)
- `ftv()` → compute and cache if None, return cached
- `apply()` → invalidate cache

### File: `infer.rs`

Change inference loop: don't apply subst to entire env. Instead, apply only to the final type before generalization:

```rust
// BEFORE:
env = env.apply(&final_subst);
let scheme = env.generalize(&final_ty);

// AFTER:
let final_ty = self_tv.apply(&final_subst);
let scheme = env.generalize(&final_ty);  // works because env entries are already generalized
```

This removes the O(N) env.apply per function entirely.

## Risk

The env.apply serves to propagate type variable resolutions. For top-level functions that are generalized, this is a no-op. But there might be edge cases:
- Impl methods that share type variables across methods
- ADT registration interleaved with functions

Mitigation: run all 760 tests. If any fail, the fix is wrong.

## Files Changed

| File | Change |
|------|--------|
| `types/src/types.rs` | Add cached_ftv to TypeEnv |
| `types/src/infer.rs` | Remove/optimize env.apply in inference loop |
