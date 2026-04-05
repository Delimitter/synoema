---
id: design
type: design
status: done
---

# Design: Doc Extraction API

## Key Decisions

- D1: Source-text scanning instead of AST modification (Comment tokens break layout/parser)
- D2: Comment attachment via proximity — no blank line between `--` and declaration
- D3: JSON output in repl via format!() (no serde in lang workspace)
- D4: MCP tool in dev_tools.rs, parses .sno at call time, no caching
- D5: File-level query only (symbol lookup deferred)
- D6: Added synoema-parser + synoema-lexer as MCP dependencies
