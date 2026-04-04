# Spec: Code Formatter (`synoema fmt`)

## CLI

```bash
synoema fmt <file.sno | directory>
synoema fmt --check <file.sno>    # exit 1 if not formatted, don't modify
```

## Rules

1. **Indentation:** 2 spaces, no tabs
2. **Trailing whitespace:** removed
3. **Blank lines:** max 1 consecutive blank line; 1 blank line between top-level decls
4. **Operator spacing:** `x + y` not `x+y`, `x == y` not `x==y`
5. **List literals:** `[1 2 3]` — single space between elements
6. **Record literals:** `{x = 1, y = 2}` — space after `=`, `, ` between fields
7. **Imports:** `import "path.sno"` grouped at top, sorted alphabetically
8. **Comments:** preserved, aligned to code
9. **Doc comments:** `---` preserved, no reformatting of content
10. **Line length:** soft limit 100 chars (no auto-wrap, warning only)

## Architecture

Parse → AST → pretty-print. Не text-based (regex/sed), а AST-based:
1. `lex + parse` → get AST with spans
2. Walk AST → emit formatted text
3. Preserve comments by attaching them to nearest AST node via spans

## Idempotency

`synoema fmt file.sno && synoema fmt file.sno` — second run is no-op.

## Что НЕ входит

- Configuration file (стиль один, не настраивается)
- Editor integration (используется через CLI)
- Range formatting (форматирует весь файл)
