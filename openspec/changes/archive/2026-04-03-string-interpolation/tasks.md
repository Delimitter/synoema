# Tasks: String Interpolation

## Checklist

- [x] T1: Add new token types (StringFragment, InterpStart, InterpEnd) to token.rs
- [x] T2: Modify scan_string in scanner.rs to handle `${` interpolation with brace depth tracking
- [x] T3: Add `\$` escape handling in scanner.rs
- [x] T4: Add lexer tests for interpolation tokenization (fragment splitting, nesting, escapes) — 9 tests
- [x] T5: Add parse_interpolated_string to parser.rs — desugar to `show` + `++`
- [x] T6: Add parser tests for desugared AST — 5 tests
- [x] T7: Add interpreter (eval) tests for string interpolation end-to-end — 8 tests
- [x] T8: Add JIT (codegen) stress tests for string interpolation — 5 tests
- [x] T9: Update GBNF grammar (synoema.gbnf)
- [x] T10: Update docs: language_reference.md (EBNF), llm/synoema.md (quick ref), PROJECT_STATE.md
- [x] T11: Verify all tests pass (771), 0 warnings
