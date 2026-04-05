# Tasks: stdlib-json-helpers

## Files

- `lang/prelude/prelude.sno` — new functions
- `lang/crates/synoema-eval/src/tests.rs` — tests
- `docs/llm/stdlib.md` — document new functions
- `docs/llm/synoema.md` — JSON pipeline example, sequencing pattern
- `lang/templates/AGENTS.md.tmpl` — update stdlib section

## Checklist

- [x] T1: Add json_str, json_int, json_arr, json_obj to prelude.sno
- [x] T2: Add intercalate to prelude.sno
- [x] T3: Add for_each to prelude.sno
- [x] T4: Add tests for all new functions
- [x] T5: Update docs/llm/stdlib.md
- [x] T6: Update docs/llm/synoema.md — JSON pipeline example + sequencing pattern
- [x] T7: Update AGENTS.md.tmpl — stdlib section + JSON example
- [x] T8: cargo test — all green
- [x] T9: Reinstall synoema, re-init bench-viz-v4, verify token count improvement
