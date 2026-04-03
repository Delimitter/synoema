# Synoema v0.1.0-alpha.1

**Released:** April 2026
**Stage:** Alpha — syntax and APIs may change. See [versioning policy](../../docs/versioning.md).

## What's in this release

- Full JIT compiler (Cranelift, x86-64) — 4.4× faster than Python
- Tree-walking interpreter — all language features
- Hindley-Milner type inference + row polymorphism + linear types
- MCP server — `eval`, `typecheck`, `run` tools; language spec resources
- Structured diagnostics — JSON + human-readable error output
- 634 tests, 0 warnings

## Language features

Integers, booleans, floats, strings, lists, list comprehensions, ranges, closures, higher-order functions (`map`/`filter`/`foldl`), records, algebraic data types, type classes (`trait`/`impl`), modules (`mod`/`use`), IO (`print`/`readline`), pattern matching (nested), tail call optimization.

## Binaries

| Platform | Directory |
|----------|-----------|
| macOS Apple Silicon | [darwin-arm64/](darwin-arm64/) |
| macOS Intel | [darwin-x64/](darwin-x64/) |
| Linux x86_64 | [linux-x64/](linux-x64/) |
| Windows x64 | [win32-x64/](win32-x64/) |

Download from [GitHub Releases](https://github.com/Delimitter/synoema/releases/tag/v0.1.0-alpha.1).

## Changelog

**0.1.0-alpha.1** (initial alpha release)
- Phase 9.2–18 complete
- All language features in JIT
- MCP server with tools + resources + prompts
- `synoema-diagnostic` crate for structured errors
