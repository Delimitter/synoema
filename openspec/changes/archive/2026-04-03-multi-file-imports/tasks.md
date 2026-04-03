# Tasks: Multi-File Imports

## Implementation Tasks

- [x] **T1: Lexer — add `KwImport` token**
  - Add `KwImport` variant to `Token` enum in `synoema-lexer/src/token.rs`
  - Add `"import" => Token::KwImport` in scanner keyword matching (`scanner.rs`)
  - Add display string in `Token::describe()` and `is_keyword()` check
  - Add lexer test for `import "file.sno"` tokenization

- [x] **T2: Parser — add `ImportDecl` and parse `import "path"`**
  - Add `ImportDecl { path: String, span: Span }` struct to `ast.rs`
  - Add `imports: Vec<ImportDecl>` field to `Program` struct
  - Parse `import "string"` at top level in `parse_program()` (before mod/use/decl)
  - Add parser test for import declaration

- [x] **T3: Import resolver — `resolve_imports()` function**
  - New function in `synoema-eval/src/lib.rs`: `pub fn resolve_imports(program: Program, base_dir: &Path) -> Result<Program, Diagnostic>`
  - Recursive loading with cycle detection (stack) and diamond caching (HashSet by canonical path)
  - Merge imported declarations (modules, decls, uses) before the importing file's own
  - New diagnostic codes: `IMPORT_CYCLE`, `IMPORT_NOT_FOUND` in `synoema-diagnostic`
  - Add ≥4 unit tests: basic import, diamond, cycle detection, file-not-found

- [x] **T4: Integration — wire resolver into eval and JIT pipelines**
  - `eval_main_inner()`: call `resolve_imports()` after parse, before `resolve_modules()`
  - `compile_and_run()`: call `resolve_imports()` after parse, before `resolve_modules()`
  - `run()`: same treatment
  - REPL `run_file` / `jit_file`: pass file's parent directory as base_dir
  - `typecheck()`: needs base_dir parameter or accept pre-resolved program
  - All existing single-file programs must work unchanged (empty imports vec)

- [x] **T5: Example files**
  - Create `examples/imports/math.sno` — module with square, cube, abs
  - Create `examples/imports/main.sno` — `import "math.sno"` + `use Math (square)` + main

- [x] **T6: GBNF grammar update**
  - Add `import_decl ::= "import" ws string_lit newline` to `tools/constrained/synoema.gbnf`

- [x] **T7: Documentation update**
  - Update `docs/llm/synoema.md` — add import syntax to quick reference
  - Update `docs/specs/language_reference.md` — add import to EBNF grammar and semantics
  - Update `context/PROJECT_STATE.md` — mention multi-file imports in "what works"

- [x] **T8: Final verification**
  - `cargo test` — all tests green, 0 warnings
  - `cargo build` — clean
  - Run `examples/imports/main.sno` in both interpreter and JIT
