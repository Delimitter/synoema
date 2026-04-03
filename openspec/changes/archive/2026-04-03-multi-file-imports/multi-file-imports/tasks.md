---
id: tasks
type: tasks
status: draft
---

# Tasks: Multi-File Imports

## Checklist

- [x] **T1: Lexer — добавить KwImport**
  - `synoema-lexer/src/scanner.rs`
  - Зарезервировать `import` как keyword
  - BPE verification: `import` = 1 token ✅
  - ~5 LOC

- [x] **T2: AST — ImportDecl + Program.imports**
  - `synoema-parser/src/ast.rs`
  - Добавить `ImportDecl { path: String, span: Span }`
  - Добавить `imports: Vec<ImportDecl>` в Program
  - ~10 LOC

- [x] **T3: Parser — parse import declarations**
  - `synoema-parser/src/parser.rs`
  - Parse `import "path.sno"` at top level (before other decls)
  - Collect into `program.imports`
  - ~30 LOC

- [x] **T4: Import resolver**
  - `synoema-eval/src/resolve.rs` (new file)
  - `resolve_imports(entry_path, source) -> Result<Program, Vec<Diagnostic>>`
  - Recursive resolution with visited + in_progress sets
  - Diamond caching (by canonical path)
  - Cycle detection (in_progress set)
  - ~100 LOC

- [x] **T5: Wire resolver into REPL**
  - `synoema-repl/src/main.rs`
  - Call `resolve_imports()` before typecheck for `run` and `jit` commands
  - Pass resolved merged Program through existing pipeline
  - ~20 LOC

- [x] **T6: Error diagnostics**
  - `synoema-diagnostic/src/lib.rs`
  - Add codes: `IMPORT_NOT_FOUND`, `IMPORT_CYCLE`
  - Error messages include file path context
  - ~15 LOC

- [x] **T7: Example files**
  - `lang/examples/imports/math.sno` — module with math functions
  - `lang/examples/imports/main.sno` — imports math.sno and uses it
  - `lang/examples/imports/diamond.sno` — diamond import pattern

- [x] **T8: Tests**
  - Parser: import declaration parsed
  - Resolver: single import works
  - Resolver: diamond import loads once
  - Resolver: circular import → error
  - Resolver: file not found → error
  - E2E: interpreter with imports
  - E2E: JIT with imports
  - ≥8 tests

- [x] **T9: Documentation (rule 7a)**
  - `docs/llm/synoema.md` — add import syntax
  - `docs/specs/language_reference.md` — §Modules section
  - `tools/constrained/synoema.gbnf` — add import_decl rule
  - `context/PROJECT_STATE.md` — update status

- [x] **T10: Final verification**
  - `cargo test` — all tests pass, 0 warnings
  - Example files run: interpreter + JIT
  - Circular import produces clean error
