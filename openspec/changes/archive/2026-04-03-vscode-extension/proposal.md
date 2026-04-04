# Proposal: Synoema VSCode Extension

## Problem Statement

While Synoema has excellent tooling via CLI (`synoema run`, `synoema jit`, `synoema eval`) and MCP integration (Claude Desktop, Cursor, Zed), developers editing `.sno` files in VSCode currently lack:

1. **Syntax highlighting** — `.sno` files appear as plain text
2. **Quick execution** — No integrated way to run code directly from the editor
3. **Inline diagnostics** — Type errors and parse errors not shown in real-time
4. **REPL integration** — No quick evaluation of expressions

## Solution Scope

Build a lightweight VSCode extension for Synoema that provides:

1. **Syntax Highlighting** — Color scheme for keywords, operators, builtins, literals, comments
2. **File type recognition** — Register `.sno` files for language features
3. **Run command** — Keybind + command palette entry to execute current file via `synoema run`
4. **JIT command** — Keybind + command palette entry to JIT-compile via `synoema jit`
5. **Eval command** — Quick eval of selected expression via `synoema eval`
6. **Output panel** — Display results and errors in VSCode's output channel

## Non-Goals (Phase 1)

- LSP (Language Server Protocol) — requires more infrastructure, defer to Phase 2
- Debugging support — breakpoints, stepping — future phase
- Package management — defer to future
- Custom themes — use standard VSCode color tokens

## Why This Matters

- **Developer experience**: Faster iteration on Synoema code within VSCode
- **Accessibility**: Lowers barrier for newcomers (no CLI needed for basics)
- **Consistency**: VSCode is where most Rust/developer tooling lives
- **Marketing**: Extension marketplace presence increases discoverability

## Success Criteria

- [x] Extension published to VSCode Marketplace
- [x] Syntax highlighting for all token types
- [x] Run/JIT/Eval commands functional with output display
- [x] README with installation + usage
- [x] Requires `synoema` CLI in PATH (or configurable path)
- [x] Tests: basic smoke tests for commands

## Timeline

Single phase: all features implemented together, tested, shipped to marketplace.
