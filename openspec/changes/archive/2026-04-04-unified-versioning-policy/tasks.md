# Tasks: Unified Versioning Policy

## Implementation

- [x] T1: Fix lang workspace version — `lang/Cargo.toml` → `0.1.0-alpha.1` + all 21 inter-crate deps
- [x] T2: Fix MCP hardcoded version — replace literal `"0.1.0"` with `env!("CARGO_PKG_VERSION")` in `mcp/synoema-mcp/src/main.rs`
- [x] T3: Fix VSCode extension version — `vscode-extension/package.json` → `0.1.0`
- [x] T4: Sync MCP Cargo workspace version — verified `mcp/Cargo.toml` is `0.1.0-alpha.1` (already correct)
- [x] T5: Update `docs/versioning.md` — added ecosystem table, single source of truth, pre-tag checklist, VSCode note
- [x] T6: Run `cargo test` from lang/ and `cargo build` from mcp/ — 875 tests pass, MCP builds clean
- [x] T7: Save versioning rules to Claude memory
