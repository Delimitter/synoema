# Proposal: Multi-File Imports

## Problem Statement

Synoema modules (`mod Math ... end`) exist only inline within a single file. There is no way to split code across files, making real projects impossible. This is a core gap for any programming language.

## Scope

- Add `import "path.sno"` syntax to load declarations from other files
- Recursive import resolution with cycle detection and diamond caching
- Works in both interpreter and JIT (merged program approach)
- 6 files modified, ~200 lines of new code, ≥8 new tests

## Prior Art

Existing archived spec: `openspec/changes/archive/2026-04-03-llm-cost-reduction-v1/specs/multi-file-imports.md`

## Success Criteria

- `import "file.sno"` loads and merges declarations
- Imported `mod` blocks accessible via `use`
- Circular imports → diagnostic error
- Diamond imports → file loaded once (cached by absolute path)
- Relative paths resolved from importing file's directory
- Works in interpreter and JIT
- GBNF grammar updated
- `docs/llm/synoema.md` updated
- Example files created
- ≥8 tests, all green, 0 warnings
