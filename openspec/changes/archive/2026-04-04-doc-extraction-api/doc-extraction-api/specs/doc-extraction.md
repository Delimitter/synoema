---
id: spec-doc-extraction
type: spec
status: done
---

# Spec: Doc Extraction API

## Capability: Comment Preservation

### Lexer behavior
- `--` followed by non-`-` character → `Token::Comment(String)` (text after `-- `, trimmed)
- `---` → `Token::DocComment(String)` (unchanged from current behavior)
- `-- ` (empty after dash-dash-space) → `Token::Comment("")`
- Comment text: strip leading `-- ` prefix, trim trailing whitespace

### Parser attachment rule
- Comment token immediately before a declaration (no blank line between) → attaches as `comment` field
- Blank line (Newline + Newline) between comment and declaration → comment discarded (separator, not annotation)
- Multiple consecutive `--` lines before a declaration → joined with `\n`
- Comments between declarations without attachment → discarded

### AST changes
```
Decl::Func     { ..., comment: Option<String> }
Decl::TypeDef  { ..., comment: Option<String> }
Decl::TraitDecl{ ..., comment: Option<String> }
Decl::TypeSig  — no change (TypeSig immediately before Func → Func gets the comment)
Decl::ImplDecl — no change
Decl::TypeAlias— no change
Decl::Test     — no change
```

### CoreIR boundary
- `comment` field stripped at desugar (same as `doc`) — 0 runtime impact

## Capability: JSON Output

### CLI
```
synoema doc <file.sno|directory> --format json
```

### Schema (per file)
```json
{
  "file": "string — filename",
  "description": "string|null — from first --- doc-comment block",
  "examples": [
    {"expr": "string", "expected": "string|null"}
  ],
  "modules": [
    {
      "name": "string",
      "doc": ["string — doc-comment lines"],
      "functions": [
        {
          "name": "string",
          "comment": "string|null — from -- comment",
          "doc": ["string — from --- doc-comment lines"],
          "line": "number"
        }
      ],
      "types": [
        {
          "name": "string",
          "comment": "string|null",
          "doc": ["string"],
          "variants": ["string"],
          "line": "number"
        }
      ]
    }
  ],
  "functions": [{ "..." : "same as modules[].functions" }],
  "types": [{ "..." : "same as modules[].types" }]
}
```

### Directory mode
Array of file objects: `[ {...}, {...} ]`

## Capability: MCP Tool

### Tool: `doc_query`

**Input:**
```json
{
  "file": "string — path relative to repo root (e.g. 'lang/examples/sorting.sno')"
}
```

**Output:** JSON as defined in schema above, truncated to ≤2000 chars (~500 tokens).

**Errors:** file not found, parse error → `isError: true` with message.

**Registration:** added to `dev_tools::tool_definitions()` and `dev_tools::call()` dispatch.
