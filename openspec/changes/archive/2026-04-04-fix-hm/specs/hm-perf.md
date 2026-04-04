# Spec: HM Type Checker Performance Fix

## Current behavior
- `infer_program` loop: for each function → `env.apply(&subst)` → `env.generalize(&ty)`
- `env.apply` walks ALL entries in env, applying substitution to each type
- `env.generalize` computes `env.ftv()` by walking all entries

## Required behavior
- Same type inference results (all 760 tests pass, zero regressions)
- Prelude with ~15 Map functions type-checks in <2s (currently 2+ min)

## Constraint
- 0 changes to inference semantics — only performance
- 0 new dependencies
