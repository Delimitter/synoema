# CLAUDE.md — Synoema Project

## Quick Reference

Synoema — programming language for LLM code generation.
7055 lines Rust, 264 tests, 7 crates, Cranelift JIT backend.

## Commands

```bash
cargo build                     # Build all
cargo test                      # Run 264 tests
cargo run -p synoema-repl -- run examples/quicksort.sno  # Interpreter
cargo run -p synoema-repl -- jit examples/factorial.sno   # JIT compile
cargo run -p synoema-repl -- eval "6 * 7"                 # Eval expression
```

## Key Files

- `context/PROJECT_STATE.md` — FULL project state, read first
- `context/DEVELOPMENT_GUIDE.md` — How to add features, roadmap, patterns
- `synoema/crates/` — All compiler source code
- `docs/articles/` — 14 articles (7 RU + 7 EN)
- `docs/research/scientific_foundations.md` — 23 verified scientific facts
- `docs/specs/language_reference.md` — Full language specification

## Architecture

```
Source (.sno) → Lexer → Parser → Types (HM) → Core IR → Eval (interpreter)
                                                       → Codegen (Cranelift JIT)
```

## Current Priorities

1. Phase 9.2: Closures in JIT (unlocks map/filter/comprehensions)
2. Phase 10.1: Tail call optimization (fixes stack overflow)
3. Phase 9.3: Strings in JIT
4. Publication: GitHub + Habr articles + HN launch

## Known Bugs

- Ackermann JIT gives 125 instead of 5 (3-equation pattern match bug in desugar)
- Euler1 stack overflow in interpreter (needs TCO)

## Rules

- Every operator MUST be exactly 1 BPE token (cl100k_base)
- Tests must pass before any commit (cargo test)
- New features: interpreter first, JIT second
- Minimal dependencies (only Cranelift + pretty_assertions)
