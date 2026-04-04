# Tasks: Tree Recursion Linearization

## Phase A: Pattern Detection

- [x] A1. `match_sub_lit(expr, var) -> Option<i64>` — matches `Sub(var, k)`
- [x] A2. `match_self_call(expr, fname, param) -> Option<i64>` — matches `f(param - k)`
- [x] A3. `detect_tree_recursion(name, body) -> Option<TreeRecPattern>` — full pattern detection

## Phase B: Code Generation

- [x] B1. Worker function: tail-recursive 3-arg function with accumulator pattern
- [x] B2. Wrapper function: forwards to worker with base values
- [x] B3. `linearize_tree_recursion(program) -> CoreProgram` — walks defs and replaces matched patterns

## Phase C: Integration

- [x] C1. Called from `optimize_program` before constant folding

## Phase D: Testing

- [x] D1. Full test suite: 960 tests pass, 0 failures, 0 new warnings
- [x] D2. JIT correctness: fib(10)=55, fib(30)=832040, fib(40)=102334155, fib(90)=2880067194370816120
- [x] D3. Performance: fib(30) from 3039ms → <1ms (>3,000,000x speedup)

## Phase E: Bug Fix (discovered during testing)

- [x] E1. Fix `is_str` to check all 3 low bits: `v & 7 == 2` instead of `v & 2 == 2`
- [x] E2. Replace all static heap thresholds with `arena_contains_ptr()` validation
- [x] E3. Fix `is_con`, `is_record`, `is_likely_list_ptr` to use arena range check
- [x] E4. All display/equality functions now immune to integer-pointer confusion
