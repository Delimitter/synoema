# Tasks: Token Savings v2

## Checklist

- [x] T1: Record punning in parser — desugar `{x, y}` to `{x = x, y = y}` in parse_atom (LBrace branch)
- [x] T2: Record pattern punning — desugar `{x, y}` in pattern position to `{x = x, y = y}`
- [x] T3: Parser tests for record punning (expr + pattern, mixed, nested) — 5 tests
- [x] T4: Eval tests for record punning end-to-end — 3 tests
- [x] T5: JIT stress tests for record punning — 2 tests
- [x] T6: Wildcard import in parser — parse `use M (*)` with Star token
- [x] T7: Wildcard import in module resolver (modules.rs) — expand `*` to all module function names
- [x] T8: Parser tests for wildcard import — 2 tests
- [x] T9: Eval tests for wildcard import end-to-end — 3 tests
- [x] T10: JIT stress tests for wildcard import — 2 tests
- [x] T11: Update GBNF grammar (record punning + wildcard import rules)
- [x] T12: Update docs: language_reference.md, llm/synoema.md
- [x] T13: Update examples: geometry.sno, modules.sno, records.sno to use new syntax
- [x] T14: Verify all tests pass (859), 0 warnings
