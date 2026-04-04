# Design: Unified Versioning Policy

## Architecture

```
SINGLE SOURCE OF TRUTH
══════════════════════

  lang/Cargo.toml
  [workspace.package]
  version = "0.1.0-alpha.1"    ◄── THE version
       │
       ├──▶ 8 lang crates        (workspace inheritance, automatic)
       │
       ├──▶ mcp/Cargo.toml       (manual sync at release)
       │    └──▶ synoema-mcp
       │         └──▶ main.rs     env!("CARGO_PKG_VERSION")
       │
       ├──▶ npm/*/package.json    (CI sets from git tag)
       │
       ├──▶ vscode-extension/     (manual sync, marketplace version)
       │    package.json
       │
       └──▶ docs + README         (manual sync)
```

## Technical Decisions

### D1: Fix MCP hardcoded version

Replace `"version": "0.1.0"` literal in `mcp/synoema-mcp/src/main.rs` with `env!("CARGO_PKG_VERSION")`.

This macro reads the version from `Cargo.toml` at compile time — zero runtime cost, always in sync.

### D2: Unify lang workspace version

Change `lang/Cargo.toml` from `version = "0.1.0"` to `version = "0.1.0-alpha.1"` to match the documented project version. All 8 crates inherit this.

### D3: VSCode extension version

VSCode Marketplace uses npm semver. Pre-release extensions use `MAJOR.MINOR.PATCH` where PATCH is odd for pre-release. However, the simplest approach at alpha stage: use `0.1.0` and bump with the ecosystem. The `-alpha.N` suffix cannot be used in marketplace versions.

Decision: `vscode-extension/package.json` → `"version": "0.1.0"` with a comment in versioning docs that marketplace strips pre-release tags.

### D4: Benchmarks excluded

`benchmarks/runner/Cargo.toml` stays at its own version. Not a user-facing component.

### D5: Update versioning.md

Expand `docs/versioning.md` with:
- Ecosystem component table
- Single source of truth declaration
- Release checklist
- VSCode marketplace note

### D6: Memory persistence

Save versioning rules to Claude memory so future conversations automatically apply them.
