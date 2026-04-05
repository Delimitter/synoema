---
id: proposal
type: proposal
status: done
---

# Proposal: Doc Extraction API — Structured Documentation from .sno Source

## Problem

Synoema .sno files contain documentation in `---` doc-comments (in AST) and `--` inline comments (stripped by lexer). Both are programmatically inaccessible: `synoema doc` outputs unstructured Markdown, MCP server has 0 tools for .sno documentation.

## Goal

1. `synoema doc file.sno --format json` — structured JSON output
2. MCP tool `doc_query` — LLM queries documentation of any .sno file
3. Extract `--` comments from source text, attach to declarations by line proximity

## Implementation

Source-text scanning approach (not AST modification): extract `--` comments from raw source during doc generation, match to declarations by line number. 0 changes to lexer/parser/layout.

## Output

- `synoema-repl/src/main.rs`: `generate_doc_json()`, `extract_source_comments()`, `find_comment_for_line()`
- `mcp/synoema-mcp/src/dev_tools.rs`: `doc_query` tool definition + implementation
- `mcp/synoema-mcp/Cargo.toml`: added synoema-parser + synoema-lexer deps
- `docs/mcp.md`, `context/PROJECT_STATE.md`, `CLAUDE.md`: updated
