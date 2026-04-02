# CLAUDE.md — Synoema Project

## Quick Reference

Synoema — programming language for LLM code generation.
~11000 lines Rust, 452 tests, 7 crates, Cranelift JIT backend.

## Commands

```bash
cargo build                     # Build all
cargo test                      # Run 452 tests
cargo run -p synoema-repl -- run examples/quicksort.sno  # Interpreter
cargo run -p synoema-repl -- jit examples/factorial.sno   # JIT compile
cargo run -p synoema-repl -- eval "6 * 7"                 # Eval expression
```

NOTE: run cargo from `lang/` subdirectory (workspace root).

## Key Files

- `context/PROJECT_STATE.md` — FULL project state, read first
- `context/DEVELOPMENT_GUIDE.md` — How to add features, roadmap, patterns
- `lang/crates/` — All compiler source code
- `docs/articles/` — 14 articles (7 RU + 7 EN)
- `docs/research/scientific_foundations.md` — 23 verified scientific facts
- `docs/specs/language_reference.md` — Full language specification

## Architecture

```
Source (.sno) → Lexer → Parser → Types (HM) → Core IR → Optimizer → Eval (interpreter)
                                                                   → Codegen (Cranelift JIT)
```

## Completed Phases

- **Phase 9.2** ✅ Closures in JIT (lambda lifting, indirect calls, map/filter)
- **Phase 9.3** ✅ Strings in JIT (tagged ptr bit 1, StrNode, show/++/length, fizzbuzz)
- **Phase 9.4** ✅ Records (interpreter + JIT: RecordNode heap, FNV-hash field access)
- **Phase 9.5** ✅ Modules (`mod Name` + `use Name (...)` — lexical namespacing, desugaring pass)
- **Phase 10.1** ✅ TCO in interpreter (iterative eval loop + 64MB stack thread)
- **Phase 10.2** ✅ Constant folding/DCE in Core IR optimizer
- **Phase 10.3** ✅ Region-based arena allocator (no malloc leaks, arena_reset after each run)
- **String ==** ✅ `synoema_val_eq` runtime dispatch — works for int and string
- **Phase 11.1** ✅ ADTs in JIT (ConNode heap alloc, tag comparison, field extraction, 6 tests)
- **Phase 11.2** ✅ Row polymorphism for records (Rémy-style row unification, 7 type tests)
- **Phase 11.3** ✅ Nested ADT patterns in JIT (nested constructor matching, 2 codegen tests)
- **Phase 11.4** ✅ Full ADT pattern matching in JIT (literal sub-patterns, triple nesting, recursive `bind_sub_pat`)
- **Phase 11.5** ✅ String literal patterns in JIT (top-level + sub-patterns inside constructors, 5 tests)
- **Phase 12a** ✅ Float in JIT (FloatNode heap-alloc, tag=0x04, 10 tests: arithmetic + comparisons + cond)
- **Phase 12b** ✅ Record patterns in JIT (CorePat::Record in compile_case + bind_sub_pat, 5 tests)
- **Phase 13** ✅ Float Eq/Ord/Show in interpreter + 19 tests (float ==, <, >, show, ADT+float)
- **`**` operator** ✅ Power operator + float math builtins (sqrt, floor, ceil, abs, round) — interpreter + JIT, 28 tests
- **VS Code extension** ✅ TextMate grammar for .sno files (`tools/vscode-extension/`)
- **Phase 14a** ✅ IO/Effects in interpreter: `()` unit, `print` (∀a. a → ()), `;` sequence op, `readline` (stdin), 11 tests
- **Phase 14b** ✅ IO in JIT: `synoema_print_val` (any tagged val via FFI), `synoema_readline` (stdin → tagged str), `Lit::Unit` → `iconst(0)`, 8 tests

## Current Priorities

1. Publication: GitHub + Habr articles + HN launch
2. Phase 15: Lists in JIT (cons, head, tail, pattern matching on lists)

## Known Bugs

- 0 warnings, 0 known bugs (452/452 tests passing)

Note: the "Ackermann JIT bug" was a false positive. `ack 3 4 = 125` is correct (2^7 − 3).

## Syntax Note: Ternary vs Cons

The `?` ternary uses `:` as else separator — SAME symbol as cons.
Cons in then-branch MUST use explicit parens: `? cond -> (x : xs) : rest`

## Rules

- Every operator MUST be exactly 1 BPE token (cl100k_base)
- Tests must pass before any commit (cargo test)
- New features: interpreter first, JIT second
- Minimal dependencies (only Cranelift + pretty_assertions)
