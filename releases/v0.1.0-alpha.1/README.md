# Synoema v0.1.0-alpha.1

**Released:** April 2026 | **Stage:** Alpha (syntax and APIs may change)

## Build

Pick your platform, then run `make` (or `.\build.ps1` on Windows):

| Platform | Directory | Build command |
|----------|-----------|---------------|
| macOS Apple Silicon | [darwin-arm64/](darwin-arm64/) | `cd darwin-arm64 && make` |
| macOS Intel | [darwin-x64/](darwin-x64/) | `cd darwin-x64 && make` |
| Linux x86_64 | [linux-x64/](linux-x64/) | `cd linux-x64 && make` |
| Windows x64 | [win32-x64/](win32-x64/) | `cd win32-x64; .\build.ps1` |

**Prerequisite:** [Rust toolchain](https://rustup.rs/) (includes `cargo`).

Each build produces two binaries: `synoema` (compiler CLI) and `synoema-mcp` (MCP server).

## Quick Start (after build)

```bash
synoema eval "6 * 7"                    # Evaluate expression → 42
synoema run examples/quicksort.sno      # Run file (interpreter)
synoema jit examples/factorial.sno      # Run file (JIT, faster)
synoema                                 # Interactive REPL
synoema test examples/                  # Run doctests
```

## What's in this release

- Full JIT compiler (Cranelift) — 4.4x faster than Python
- Tree-walking interpreter — all language features
- Hindley-Milner type inference + row polymorphism + linear types
- MCP server — `eval`, `typecheck`, `run` tools
- Structured diagnostics — JSON + human-readable errors
- 890+ tests, 0 warnings

## Language features

Integers, booleans, floats, strings, lists, list comprehensions, ranges, closures, higher-order functions (`map`/`filter`/`foldl`), records, algebraic data types, type classes (`trait`/`impl`), modules (`mod`/`use`), IO (`print`/`readline`), pattern matching (nested), tail call optimization, Result type with combinators.

## Alternative: Download pre-built binaries

Download from [GitHub Releases](https://github.com/Delimitter/synoema/releases/tag/v0.1.0-alpha.1).

See [versioning policy](../../docs/versioning.md) for version guarantees.
