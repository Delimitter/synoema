# Testing

Synoema is covered by 998 tests across 8 crates. All tests pass with `0 failures, 0 warnings`.

## Quick Start

```bash
cd lang/
cargo test        # all tests — 998/998 green
```

## Test Structure

### Unit tests

Each crate contains integration tests in `src/tests.rs`:

| Crate | Tests | What is tested |
|-------|------:|----------------|
| `synoema-lexer` | 97 | Tokenization, offside rule, escape sequences, string interpolation |
| `synoema-parser` | 90 | Pratt parser, type aliases, imports, error recovery, string interp |
| `synoema-types` | 90 | Hindley-Milner, row polymorphism, type classes, alias expansion |
| `synoema-core` | 60 | Core IR, desugaring, optimizations |
| `synoema-eval` | 294 | Tree-walking interpreter, all language features |
| `synoema-codegen` | 225 | Cranelift JIT — arithmetic, strings, ADTs, closures |
| `synoema-diagnostic` | 29 | Diagnostic messages, LLM hints, error formatting |
| `synoema-repl` | 28 | CLI commands, project scaffolding |
| **Total** | **998** | |

### Stress tests

Stress tests live in `tests/stress.rs` of each crate and verify performance and stability under heavy load:

```bash
# Run stress tests for a specific crate
cargo test --test stress -p synoema-lexer   -- --nocapture
cargo test --test stress -p synoema-types   -- --nocapture
cargo test --test stress -p synoema-eval    -- --nocapture
cargo test --test stress -p synoema-codegen -- --nocapture
```

| Crate | Stress tests | Examples |
|-------|-------------|----------|
| `synoema-lexer` | 10 (+ 3 ignored) | 100K tokens, deep nesting |
| `synoema-types` | 9 (+ 2 ignored) | 500 functions, 100 ADT variants |
| `synoema-eval` | 17 (+ 6 ignored) | fib(25), sorting 10K, typeclass dispatch |
| `synoema-codegen` | 49 (+ 9 ignored) | fib(35) via JIT, 1K iterations map/filter |

Tests marked with `#[ignore]` require the `--ignored` flag and may take a long time:

```bash
cargo test --test stress -p synoema-eval -- --ignored --nocapture
```

### Ollama-gated tests

The benchmark runner includes tests that require a local ollama installation. These are `#[ignore]`d by default:

```bash
cd benchmarks/runner
cargo test -- --ignored --nocapture    # runs ollama detection + model pull + single task
```

These tests check ollama availability, auto-pull `qwen3:8b`, and run a single LLM generation task locally. Requires ollama installed and running (`ollama serve`).

## Built-in Language Tests

Synoema supports three kinds of tests in `.sno` files:

### Doctests
```
--- example: fact 5 == 120
fact n = ? n == 0 -> 1 : n * fact (n - 1)
```

### Test declarations
```
test "fact base" = fact 0 == 1
test "sort then reverse" = reverse (qsort [3 1 2]) == [3 2 1]
```

### Property-based tests
```
test "reverse involution" = prop xs -> reverse (reverse xs) == xs
test "fact positive" = prop n -> fact n > 0 when n >= 0 && n <= 10
```

### Running

```bash
cargo run -p synoema-repl -- test examples/testing.sno          # single file
cargo run -p synoema-repl -- test examples/                     # directory
cargo run -p synoema-repl -- test examples/ --filter "sort"     # filter by name
```

Keywords: `test` (declaration), `prop` (property generator), `when` (conditional property).
All three are exactly 1 BPE token in cl100k_base.

## Running by Crate

```bash
# Single crate
cargo test -p synoema-lexer
cargo test -p synoema-parser
cargo test -p synoema-types
cargo test -p synoema-core
cargo test -p synoema-eval
cargo test -p synoema-codegen

# With output (don't swallow println!)
cargo test -p synoema-eval -- --nocapture

# Specific test
cargo test -p synoema-eval -- test_factorial

# Single-threaded (for deterministic output)
cargo test -p synoema-eval -- --test-threads=1
```

## Interactive Dashboard

For a visual test runner in the browser, use the Synoema-powered server:

```bash
cd lang/
cargo run -p synoema-repl -- run examples/stress_server.sno
# Open: http://localhost:8765/stress_tests.html
```

The dashboard shows results in real time via SSE streaming. Details: [docs/stress-server.md](stress-server.md).

## Performance (release vs debug)

Tests run in debug mode by default, which is ~10x slower than release:

```bash
# Debug (default)
cargo test

# Release
cargo test --release
```

Stress tests with strict time constraints are marked `#[cfg(not(debug_assertions))]` and are skipped in debug builds.

## Rules for New Tests

- `cargo test` must be clean (0 failures, 0 warnings) before every commit
- New features: test in interpreter first, then in JIT
- Stress tests that overflow the stack in debug (> ~1500 levels of recursion) are marked `#[ignore]`
- Tests with performance assumptions are wrapped in `#[cfg(not(debug_assertions))]`

## CI

```bash
# CI command (equivalent to cargo test)
cargo test 2>&1 | grep -E "test result|FAILED"
```

Expected output: lines like `test result: ok. N passed; 0 failed`.
