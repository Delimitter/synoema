# Contributing to Synoema

Thank you for your interest in contributing to Synoema! Whether you're fixing a bug, proposing a feature, improving documentation, or adding benchmarks, your contribution is welcome.

Before contributing, please read this guide and our [Code of Conduct](CODE_OF_CONDUCT.md).

## Quick Start

```bash
git clone https://github.com/Delimitter/synoema
cd synoema/lang
cargo build            # Build all crates
cargo test             # Run 998 tests (must be 0 failures, 0 warnings)
```

**Prerequisites:** Rust stable ≥ 1.75 ([rustup.rs](https://rustup.rs)). No other dependencies — only Cranelift (JIT) and pretty_assertions (tests).

Try running a program:

```bash
cargo run -p synoema-repl -- run examples/quicksort.sno    # [1 2 3 4 5 6 7 8 9]
cargo run -p synoema-repl -- jit examples/factorial.sno    # 3628800
cargo run -p synoema-repl -- eval "6 * 7"                  # 42
```

## Architecture

```
Source → Lexer → Parser → Type Check → Core IR ─┬→ Interpreter
  .sno                                  System F  │    All features, closures, IO
        Offside  Pratt   HM infer.               │
        rule     parser  Algorithm W             └→ Cranelift JIT
                                                       Native x86-64, 4.4× faster
```

### Crates

| Crate | Lines | Tests | Purpose |
|-------|------:|------:|---------|
| `synoema-lexer` | ~800 | 82 | Tokenization + offside rule (INDENT/DEDENT) |
| `synoema-parser` | ~1700 | 43 | Pratt parser, 15 ExprKind variants |
| `synoema-types` | ~1900 | 61 | Hindley-Milner inference + row polymorphism |
| `synoema-core` | ~1600 | 44 | Core IR (System F) + desugaring + optimizer |
| `synoema-eval` | ~1900 | 137 | Tree-walking interpreter |
| `synoema-codegen` | ~3100 | 191 | Cranelift JIT compiler + runtime |
| `synoema-diagnostic` | ~400 | — | Structured errors (human + JSON) |
| `synoema-repl` | ~300 | — | CLI: run / jit / eval / build / test / REPL |

Data flows through the pipeline: `.sno` source → tokens → AST → typed AST → Core IR → interpreter or JIT-compiled native code.

## Project Structure

```
synoema/
├── lang/                          # Compiler workspace (Rust)
│   ├── crates/                    # All compiler crates (see table above)
│   ├── examples/                  # Example programs (.sno)
│   ├── prelude/                   # Standard prelude (Result, Map, etc.)
│   ├── templates/                 # Project scaffolding templates
│   └── tools/constrained/         # GBNF grammar for constrained decoding
├── docs/                          # Documentation
│   ├── LANGUAGE.md                # Language guide for users
│   ├── install.md                 # Installation guide
│   ├── llm/                       # LLM-optimized reference
│   ├── specs/                     # Formal language specification
│   ├── articles/                  # Educational articles (EN + RU)
│   └── research/                  # Scientific foundations
├── context/                       # Internal context for Claude agent
├── mcp/                           # MCP server (separate workspace)
├── releases/                      # Platform-specific build scripts
├── vscode-extension/              # VS Code extension
├── CLAUDE.md                      # Project rules and entry point
├── CONTRIBUTING.md                # This file
└── README.md                      # User-facing readme
```

## Running Tests

```bash
cd lang

# All tests (must pass before any commit)
cargo test

# Single crate
cargo test -p synoema-lexer
cargo test -p synoema-codegen

# Single test by name
cargo test -p synoema-eval -- test_name

# Stress tests
cargo test --test stress -p synoema-codegen

# In-language doctests and unit tests
cargo run -p synoema-repl -- test examples/testing.sno

# Stress test dashboard (HTTP)
cargo run -p synoema-repl -- run examples/stress_server.sno
# → http://localhost:8765/stress_tests.html
```

Full testing guide: [docs/testing.md](docs/testing.md)

## Running Benchmarks

**Prerequisites:** `pip3 install tiktoken` (token counting). For LLM tests: `pip3 install openai` + [OpenRouter API key](https://openrouter.ai). For runtime: `node`, `g++` in PATH, and `cd lang && cargo build --release -p synoema-repl`.

```bash
cd benchmarks

# Token efficiency + runtime (no API key needed)
cargo run --manifest-path runner/Cargo.toml -- run --phases token,runtime

# Full suite including LLM code generation
cargo run --manifest-path runner/Cargo.toml -- run --all --openrouter-key YOUR_KEY
```

Run benchmarks when adding features to check for performance regression. Results go to `benchmarks/results/`. Full CLI reference and troubleshooting: [docs/benchmarks.md](docs/benchmarks.md)

## Adding a Feature

### Workflow

1. **Write tests first** — add test cases for the new behavior
2. **Implement in interpreter** — `synoema-eval` is the reference implementation
3. **Implement in JIT** — `synoema-codegen` for performance (if applicable)
4. **Check BPE alignment** — any new operator must be exactly 1 BPE token (cl100k_base)
5. **Update GBNF grammar** — `lang/tools/constrained/synoema.gbnf`
6. **Run all tests** — `cargo test` must show 0 failures, 0 warnings

### JIT pattern: Runtime FFI

New JIT capabilities are added via `extern "C"` functions in `runtime.rs`, registered in `compiler.rs`:

```rust
// runtime.rs
#[no_mangle]
pub extern "C" fn rt_my_function(arg: i64) -> i64 {
    // implementation
}

// compiler.rs — register in build_runtime_imports()
```

### Key constraints

- **Tagged pointer ABI** — JIT values use tagged i64 (bit 0 = list, bit 1 = string, tag-bytes for ADT/float/record). Do not break the ABI.
- **Arena allocator** — all JIT allocations go through the arena. `arena_reset` after each run.
- **No new dependencies** — only Cranelift and pretty_assertions are allowed.
- **Idiomatic Rust** — no `unsafe` except FFI in `runtime.rs`.

## How to Contribute

### Bug Reports

Use the [bug report template](.github/ISSUE_TEMPLATE/bug_report.md) on GitHub. Include:
- Synoema version (`synoema --version`)
- Operating system
- Minimal `.sno` reproduction
- Expected vs actual behavior
- Full compiler error output

### Feature Proposals (RFC)

Create a GitHub issue with the `[RFC]` prefix in the title. Include:
- Summary of the proposed feature
- Motivation and use cases
- Proposed syntax (if language feature)
- Impact on token efficiency (every feature must justify its token cost)
- Alternatives considered

### Code Contributions

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/my-feature`)
3. Make your changes following the workflow above
4. Ensure all checks pass (`cargo test`, `cargo clippy`)
5. Submit a pull request with signed-off commits

## Code Standards

- `cargo fmt` before every commit — no formatting violations
- `cargo clippy` with zero warnings
- `cargo test` must pass with 0 failures, 0 warnings
- Every new feature requires tests (interpreter AND JIT where applicable)
- Compiler error messages must include source location and hint
- Every operator must be exactly 1 BPE token (cl100k_base)
- Commit format: `feat: description` or `fix: description`

## VS Code Extension

```bash
cd vscode-extension
./install.sh          # build + install in one step
./install.sh --keep   # keep node_modules and dist after install
```

Requires Node.js and `code` CLI in PATH. The script handles `npm install`, esbuild, vsce packaging, and `code --install-extension` automatically. Re-run to update.

See [vscode-extension/DEVELOPMENT.md](vscode-extension/DEVELOPMENT.md) for development workflow (F5 debug, adding commands, grammar updates).

## Building Releases

```bash
cd releases/v0.1.0-alpha.2

# macOS Apple Silicon
cd darwin-arm64 && make

# macOS Intel
cd darwin-x64 && make

# Linux
cd linux-x64 && make

# Windows
cd win32-x64 && .\build.ps1
```

Each build produces `synoema` (compiler CLI) and `synoema-mcp` (MCP server).

## Developer Certificate of Origin (DCO)

We use the Developer Certificate of Origin (DCO) instead of a Contributor License Agreement (CLA). By contributing, you certify that you have the right to submit the work under the project's license.

**Sign off every commit:**

```bash
git commit -s -m "Add parser for let-expressions"
```

This adds a `Signed-off-by: Your Name <email>` line to the commit message. All commits in a pull request must be signed off.

**Why DCO over CLA?** DCO is lighter-weight, does not require signing a separate legal agreement, is used by the Linux kernel and many major projects, and does not transfer rights away from the contributor.

<details>
<summary>Full text of the Developer Certificate of Origin 1.1</summary>

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.

Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```

</details>

## License Headers

All new source files must include the appropriate SPDX license header.

### Rust files in `lang/crates/` (EXCEPT `synoema-codegen/`)

```rust
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors
```

### Rust files in `lang/crates/synoema-codegen/`

```rust
// SPDX-License-Identifier: BUSL-1.1
// Copyright (c) 2025-present Andrey Bubnov
```

### Files in `tools/`

```python
# SPDX-License-Identifier: BUSL-1.1
# Copyright (c) 2025-present Andrey Bubnov
```

### Example files in `examples/`

```
-- SPDX-License-Identifier: MIT-0
```

**Note:** BSL-licensed files use the author's name (the licensor), not "Synoema Contributors", because BSL commercial licensing authority rests with the licensor.

Run `scripts/add_headers.sh` to automatically add missing headers.

## Licensing Summary

| Directory | License | SPDX |
|-----------|---------|------|
| `lang/crates/` (except codegen) | Apache-2.0 | `Apache-2.0` |
| `lang/crates/synoema-codegen/` | BSL-1.1 | `BUSL-1.1` |
| `tools/` | BSL-1.1 | `BUSL-1.1` |
| `spec/` | CC-BY-SA-4.0 | `CC-BY-SA-4.0` |
| `docs/` | CC-BY-SA-4.0 | `CC-BY-SA-4.0` |
| `examples/` | MIT-0 | `MIT-0` |

## Roadmap

- [x] Full language: functions, ADTs, type classes, modules, IO, pattern matching
- [x] Cranelift JIT — native x86-64, full feature parity with interpreter
- [x] Structured diagnostics — JSON + human-readable errors
- [x] MCP server — eval, typecheck, run tools
- [x] Constrained decoding — GBNF grammar
- [x] 998 tests, 0 warnings
- [ ] **LSP server** — autocomplete, go-to-definition, inline errors
- [ ] **Web playground** — WASM-compiled interpreter in the browser
- [ ] **LLVM backend** — `--backend llvm` for maximum optimization

## Code of Conduct

This project follows the [Contributor Covenant v2.1](CODE_OF_CONDUCT.md). Please read it before participating.

## Questions?

Open a GitHub Discussion or reach out at andbubnov@gmail.com.
