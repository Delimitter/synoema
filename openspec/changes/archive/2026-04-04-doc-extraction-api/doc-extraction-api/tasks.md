---
id: tasks
type: tasks
status: done
---

# Tasks: Doc Extraction API

## Design change: source-text scanning instead of AST modification

Original design called for `Token::Comment` in lexer + `comment: Option<String>` in AST.
Reverted: Comment tokens break layout pass and parser (comments appear inline after expressions).
Simpler approach: extract `--` comments from raw source text during doc generation, match to declarations by line proximity. 0 changes to lexer/parser/layout.

## Layer 1: JSON Doc Renderer (CLI)

- [x] **T1: JSON renderer with comment extraction**
  - `synoema-repl/src/main.rs`: `generate_doc_json(path)` function
  - `extract_source_comments(source)` — extracts `--` comments by line number
  - `find_comment_for_line()` — attaches comments to declarations by proximity (no blank line rule)
  - `generate_docs()` routes to JSON or MD based on `--format` flag
  - JSON schema: `{file, description, examples, modules, functions, types}` per spec
  - Each function/type includes `comment` (from `--`) and `doc` (from `---`)
  - 0 changes to lexer, parser, layout pass, or downstream crates

## Layer 2: MCP Tool

- [x] **T2: MCP tool `doc_query`**
  - `mcp/synoema-mcp/Cargo.toml`: added `synoema-parser` + `synoema-lexer` dependencies
  - `mcp/synoema-mcp/src/dev_tools.rs`: tool definition, dispatch, implementation
  - Same source-text scanning approach as repl JSON renderer
  - Output ≤2000 chars (~500 tokens)
  - 3 tests: missing file arg, nonexistent file, valid file with assertions

## Documentation

- [x] **T3: Update documentation (Rule 7a)**
  - `docs/mcp.md`: added `doc_query` to Dev Intelligence Tools table
  - `context/PROJECT_STATE.md`: added Doc Extraction API to features
  - `CLAUDE.md`: updated status line
