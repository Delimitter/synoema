---
id: proposal
type: proposal
status: done
---

# Proposal: Doc Extraction API — Structured Documentation from .sno Source

## Problem

Synoema source files (.sno) contain two layers of documentation embedded in code:

1. **Doc-comments (`---`)** — already parsed into AST (`Decl.doc: Vec<String>`), contains module descriptions, examples, guide metadata
2. **Inline comments (`--`)** — stripped by lexer, contains per-function descriptions ("Merge two sorted lists", "DFS: check if target is reachable"), design decisions, algorithmic notes

**Current state:** both layers are inaccessible programmatically:
- `synoema doc` outputs unstructured Markdown to stdout — no JSON, no filtering
- MCP server has 0 tools for .sno documentation (all dev_tools are for Rust source, not Synoema code)
- `synoema://examples/{name}` resource returns raw source — LLM must parse comments itself

**Impact on LLM-optimization (Rule 1):**
- LLM generating Synoema code cannot query "what does function X do?" via MCP
- LLM must load full source into context (expensive) instead of structured summary (cheap)
- 62 doc-annotations + 154 inline comments across examples — all invisible to MCP consumers
- Token budget: raw source ~300-500 tok/file vs structured JSON ~200-350 tok/file

**Measured gap in MCP:**
- 11 tools + 4 resources in MCP server
- 0 of them provide access to doc-annotations inside .sno files
- `crate_info`, `file_summary` — for Rust compiler code, not Synoema user code

## Goal

Make all documentation embedded in .sno source extractable via CLI and MCP:

### 1. Preserve `--` comments in AST
Lexer emits `Token::Comment(String)` instead of discarding. Parser attaches to nearest declaration (same proximity rule as `---`).

### 2. JSON structured output
`synoema doc file.sno --format json` returns structured data:
```json
{
  "file": "sorting.sno",
  "description": "Sorting: mergesort and insertion sort.",
  "examples": [{"expr": "main", "expected": "()"}],
  "functions": [
    {"name": "insert", "comment": "Insert into sorted list", "doc": []},
    {"name": "msort", "comment": "Merge sort", "doc": []}
  ],
  "types": []
}
```

### 3. MCP tool `doc_query`
LLM queries documentation of any .sno file or symbol through MCP:
```
tool: doc_query
args: { "file": "examples/sorting.sno" }
```

## Constraints

### Rule 5: Minimalism
- 0 new dependencies (no serde — MCP already uses serde_json)
- 0 new crates — extend existing files
- ~140 LOC total delta (lexer ~40, parser ~20, repl ~80)

### Rule 1: LLM-optimization / Rule 7b: Token budget
- MCP tool response ≤500 tokens per file
- JSON format: tables, not prose

### Rule 3: Performance
- Comment preservation: lexer already checks `--`, just stores instead of discarding
- Runtime: 0 impact (comments stripped at CoreIR boundary like doc-comments)

### Rule 2: Reliability
- Tests for comment lexing, attachment, JSON output
- `cargo test` clean before commit

## Non-goals

- HTML documentation renderer
- `--` comments as structured metadata (params, returns) — types document this better
- Search across multiple files (future: `doc_query` with `symbol` param)
- Modifying existing `---` doc-comment behavior

## Output

### Modified files
| File | Change |
|------|--------|
| `synoema-lexer/src/scanner.rs` | `--` → `Token::Comment(String)` instead of skip |
| `synoema-lexer/src/token.rs` | Add `Comment(String)` variant |
| `synoema-parser/src/ast.rs` | Add `comment: Option<String>` to Decl variants |
| `synoema-parser/src/parser.rs` | Collect Comment tokens, attach to next Decl |
| `synoema-repl/src/main.rs` | `--format json` in `synoema doc`, JSON renderer |
| `mcp/synoema-mcp/src/tools.rs` | Register `doc_query` tool |
| `mcp/synoema-mcp/src/dev_tools.rs` | Implement `doc_query` |

### Documentation updates (Rule 7a)
- `docs/mcp.md` — add `doc_query` tool
- `context/PROJECT_STATE.md` — update features
- `CLAUDE.md` — update status
