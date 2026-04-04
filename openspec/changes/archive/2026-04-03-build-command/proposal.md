# Proposal: Build Command for Synoema Projects

## Problem Statement

Currently, Synoema provides subcommands for `init`, `run`, `jit`, `eval`, and `test`, but there is no dedicated `build` command to compile a Synoema project. Users must use `cargo build` to build the Rust project itself, but cannot build individual `.sno` files or projects into portable artifacts.

## Solution Overview

Add a `build` subcommand to the Synoema REPL (`synoema-repl`) that:

1. **Compiles a `.sno` file or project** to bytecode (Core IR or native binary via JIT)
2. **Generates output artifacts** (`.sno.bc` bytecode or `.sno` executable)
3. **Validates the program** (type checking, optimization, diagnostics)
4. **Supports incremental builds** via a simple manifest or caching strategy

## Scope

### In Scope
- Add `build` subcommand to CLI
- Output bytecode to `<filename>.bc`
- Full diagnostic reporting (errors, warnings)
- Support for multi-file projects (via imports)

### Out of Scope
- Packaging/distribution (reserved for future `package` command)
- Cross-compilation or platform-specific binaries
- Incremental rebuild tracking beyond single file

## Success Criteria

1. `synoema build examples/quicksort.sno` produces `examples/quicksort.sno.bc`
2. Build output includes all compilation phases (parse, type-check, optimize, codegen)
3. Errors are reported with context and suggestions
4. Tests pass (100% existing test coverage maintained)

## Dependencies

- Uses existing compiler infrastructure (lexer, parser, type checker, codegen)
- No new crates or external dependencies required
- Integrates with `synoema-diagnostic` for error reporting
