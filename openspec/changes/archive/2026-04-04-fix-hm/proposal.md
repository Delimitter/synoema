# Proposal: Fix HM Type Checker Performance

## Problem
Adding ~15 polymorphic Map functions to prelude causes type checking to take 2+ minutes (should be <1s). Root cause: O(N²) behavior in the inference loop from `env.apply()` and `env.generalize()` after each top-level function.

## Root Cause Analysis
1. `env.apply(&final_subst)` walks entire env after each function — O(N) per function = O(N²) total
2. `env.generalize()` calls `env.ftv()` which walks all env entries — O(N) per function = O(N²) total
3. `subst.compose()` re-applies substitution to all values — chains grow with each function
4. For generalized schemes in env, apply is a no-op (quantified vars aren't affected by external substitutions)

## Fix
1. Skip `env.apply` for already-generalized schemes — only apply to current function's entry
2. Cache `ftv` on TypeEnv — compute incrementally on insert/remove

## Not in Scope
- Union-find unification (major rewrite)
- Lazy instantiation
- Changing the HM algorithm fundamentally
