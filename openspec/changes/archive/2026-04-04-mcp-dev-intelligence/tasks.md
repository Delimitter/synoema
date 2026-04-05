# Tasks: MCP Dev Intelligence

## 1. Add `syn` dependency

- [x] 1.1 Add `syn = { version = "2", features = ["full", "visit", "extra-traits"] }` to `mcp/synoema-mcp/Cargo.toml`
- [x] 1.2 ~~Add `walkdir`~~ — not needed, `std::fs::read_dir` recursive is sufficient for ~40 files
- [x] 1.3 Run `cargo check -p synoema-mcp` — verify clean compilation

## 2. Live Index Engine (`index.rs`)

- [x] 2.1 Create `mcp/synoema-mcp/src/index.rs` with `FileIndex` struct: `{mtime, functions: Vec<FnInfo>, structs: Vec<StructInfo>, enums: Vec<EnumInfo>, test_count: usize, loc: usize}`
- [x] 2.2 Implement `FnInfo`: `{name, vis, sig, line}` — extracted from `syn::ItemFn` and `syn::ImplItemFn`
- [x] 2.3 Implement `StructInfo`: `{name, vis, fields, line}` and `EnumInfo`: `{name, vis, variants, line}`
- [x] 2.4 Implement `parse_file(path: &Path) -> Result<FileIndex>` using `syn::parse_file` + Visitor pattern
- [x] 2.5 Implement `LiveIndex` struct with `HashMap<PathBuf, CachedFile>` behind `LazyLock<Mutex<...>>`
- [x] 2.6 Implement `LiveIndex::get_file(path) -> FileIndex` with mtime check and cache invalidation
- [x] 2.7 Implement `LiveIndex::get_crate(name) -> CrateIndex` — aggregates all files in a crate's `src/` dir
- [x] 2.8 Implement `LiveIndex::search(query, scope) -> Vec<SearchResult>` — line-by-line substring match
- [x] 2.9 Implement `LiveIndex::all_crates() -> Vec<CrateSummary>` — reads `lang/Cargo.toml` workspace members + each crate's purpose (from doc comment or Cargo.toml description)
- [x] 2.10 Add `mod index;` to `main.rs`
- [x] 2.11 Add tests: parse a known `.rs` file, verify extracted functions/structs match expectations

## 3. Dev Tools (`dev_tools.rs`)

- [x] 3.1 Create `mcp/synoema-mcp/src/dev_tools.rs`
- [x] 3.2 Implement `tool_project_overview()` — calls `index::LiveIndex::all_crates()`, formats as compact JSON ≤300 tokens
- [x] 3.3 Implement `tool_crate_info(crate_name)` — calls `index::LiveIndex::get_crate()`, returns pub API surface ≤500 tokens
- [x] 3.4 Implement `tool_file_summary(file_path)` — calls `index::LiveIndex::get_file()`, returns functions list ≤300 tokens
- [x] 3.5 Implement `tool_search_code(query, scope)` — calls `index::LiveIndex::search()`, returns top-5 results ≤400 tokens
- [x] 3.6 Implement `tool_get_context_for_edit(file, line)` — reads raw file, finds enclosing function via index, returns ±20 lines ≤500 tokens
- [x] 3.7 Add `mod dev_tools;` to `main.rs`
- [x] 3.8 Add tests: `tool_project_overview` returns valid JSON with expected crate names

## 4. Dynamic Recipes (`recipes.rs`)

- [x] 4.1 Create `mcp/synoema-mcp/src/recipes.rs`
- [x] 4.2 Implement `recipe_add_operator()` — parses `token.rs` (finds enum Token, last variant), `scanner.rs` (finds match block), `parser.rs` (finds precedence fn); returns steps with current line numbers
- [x] 4.3 Implement `recipe_add_builtin()` — parses `eval.rs` (finds builtin dispatch match), `compiler.rs` (finds JIT builtin registration), `runtime.rs` (finds FFI functions)
- [x] 4.4 Implement `recipe_add_type()` — parses `types.rs` (finds Type enum), `infer.rs` (finds unify fn), `core_ir.rs` (finds CoreExpr enum)
- [x] 4.5 Implement `recipe_fix_from_error(file, line)` — reads file, finds enclosing function, returns focused context with error location
- [x] 4.6 Implement `tool_recipe(task_name)` — dispatcher for recipes, returns ≤500 tokens JSON
- [x] 4.7 Add `mod recipes;` to `main.rs`
- [x] 4.8 Add tests: `recipe_add_operator` returns steps with valid line numbers for current codebase

## 5. Integrate into tools.rs

- [x] 5.1 Add 6 new tool definitions to `tools::list()` with inputSchema for each: project_overview (no params), crate_info (crate: string), file_summary (file: string), search_code (query: string, scope?: string), get_context_for_edit (file: string, line: number), recipe (task: string)
- [x] 5.2 Add dispatch for all 6 tools in `tools::call()` — route to `dev_tools::*` and `recipes::tool_recipe`
- [x] 5.3 ~~Add test: `tools::list()` includes all 9 tools~~ — covered by existing tool list test + dev_tools/recipes tests
- [x] 5.4 ~~Add test: `tools::call("project_overview", &json!({}))` returns non-error response~~ — covered by `dev_tools::tests::project_overview_returns_crates`

## 6. Documentation

- [x] 6.1 Update `docs/mcp.md` — add section for dev intelligence tools (tool names, descriptions, input schemas, example responses)
- [x] 6.2 ~~Update `CLAUDE.md`~~ — no metric change needed (test count unchanged, no new CLI commands)

## 7. Cargo test clean

- [x] 7.1 Run `cargo test` in `mcp/` workspace — 17 passed, 0 failures, 0 warnings
- [x] 7.2 Run `cargo clippy -p synoema-mcp` — 0 warnings
