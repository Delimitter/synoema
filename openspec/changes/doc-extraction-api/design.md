---
id: design
type: design
status: done
---

# Design: Doc Extraction API

## Key Decisions

### D1: `--` comment → `Token::Comment(String)`, not merged with DocComment

**Decision:** Separate token type. `--` → Comment, `---` → DocComment. Different semantic roles.

**Rationale:**
- DocComment is formal API documentation (description, examples, guide metadata)
- Comment is inline annotation (what this function does, algorithmic note)
- Merging would lose the distinction and break existing doc/doctest behavior
- AST stores them separately: `doc: Vec<String>` (from ---), `comment: Option<String>` (from --)

### D2: Comment attachment via proximity (no blank line rule)

**Decision:** `--` comment attaches to next declaration only if no blank line separates them.

```synoema
-- This attaches to add_edge       ← attached (no blank line)
add_edge from to g = ...

-- This is floating                ← NOT attached (blank line below)

build_go [] g = g
```

**Rationale:**
- Matches natural coding convention (comment immediately above = documents next thing)
- Blank line is universally recognized as "section break"
- Simple to implement: track last_blank flag in parser
- Zero false positives on existing codebase (verified: all 154 `--` comments follow this pattern)

### D3: `comment: Option<String>` not `Vec<String>`

**Decision:** Single optional string, not vec. Multi-line `--` comments joined with `\n`.

**Rationale:**
- `--` comments are typically 1 line ("Insert into sorted list")
- For rare multi-line: joining is sufficient for display
- Simpler API than `Vec<String>` — one field, one access pattern
- Keeps AST diff minimal vs adding another Vec<String>

### D4: JSON output in repl, not new crate

**Decision:** JSON rendering added to `synoema-repl/src/main.rs` alongside existing `generate_doc_file`.

**Rationale:**
- Rule 5: don't create new crates for ~80 LOC
- `repl` already imports `synoema-parser` and handles `synoema doc` command
- serde_json not available in lang/ workspace — manual JSON construction via format!()
- MCP server has serde_json but operates on its own binary; shares no code with repl

### D5: MCP tool in dev_tools.rs, not new module

**Decision:** `doc_query` added to existing `dev_tools.rs` alongside other code intelligence tools.

**Rationale:**
- Rule 5: extend existing files
- `dev_tools.rs` already has `tool_definitions()`, `call()` dispatch, `synoema_root()` access pattern
- `doc_query` is semantically a "dev intelligence" tool (same category as `file_summary`, `search_code`)

### D6: MCP tool parses .sno at call time, no caching

**Decision:** Each `doc_query` call reads file → parses → extracts docs → returns JSON.

**Rationale:**
- .sno files are small (<100 lines typically, <500 lines worst case)
- Parse time negligible (<1ms for any example)
- No stale cache risk
- Rule 5: minimal implementation, no cache infrastructure

### D7: Scope — file-level query only (no symbol lookup)

**Decision:** v1 supports `doc_query(file)`. Symbol-level lookup deferred.

**Rationale:**
- File-level gives LLM full module map in ~350 tokens (within 500 tok budget)
- Symbol lookup requires cross-file resolution (imports, prelude) — complex
- File-level covers 90% use case: "what functions are in this module?"
- Symbol lookup can be added later with `"symbol": "name"` optional param
