# Tasks: TCO in JIT

- [x] Add `TcoContext` struct and thread `tco_ctx: Option<&TcoContext>` parameter through `compile_expr` and all call sites
- [x] Propagate `tco_ctx` in tail positions: function body, Case branches, Let/LetRec body; pass `None` for non-tail positions
- [x] Detect self-tail-calls in App handler: when `flatten_apps` returns `self_name` and `tco_ctx` is Some, emit param reassignment + jump instead of call
- [x] Store param Variables in `TcoContext` during function definition, record entry block
- [x] Add tests: deep tail recursion (countdown 1M, sum_to 1M), verify factorial/gcd unchanged
- [x] Update docs: PROJECT_STATE.md, PHASES.md (TCO in JIT note)
