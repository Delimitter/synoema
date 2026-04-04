# Proposal: Unified Versioning Policy

## Problem

Ecosystem has 4 distributable components (lang CLI, MCP server Cargo, MCP npm, VSCode extension) with versions that diverged:

- Lang workspace: `0.1.0` (no pre-release tag)
- MCP Cargo workspace: `0.1.0-alpha.1`
- MCP npm main: `0.1.0-alpha.4` (drifted from Cargo)
- MCP npm platforms: `0.1.0-alpha.1`
- VSCode extension: `0.0.1`
- MCP hardcoded in main.rs: `"0.1.0"` (mismatch with Cargo.toml)
- Benchmarks runner: `0.1.0` (internal, not part of policy)

No single source of truth. No mechanism for atomic version bumps. Users cannot determine compatibility between components.

## Decision

**Monolithic versioning (Approach A)**: all user-facing ecosystem components share one version number derived from a single source of truth. Internal tools (benchmarks) excluded.

## Scope

1. Unify all user-facing versions to a single version
2. Establish a single source of truth for the version
3. Fix all mismatches (hardcoded version in MCP, lang workspace missing pre-release tag)
4. Update `docs/versioning.md` with ecosystem-wide version matrix and rules
5. Document per-change version directory naming convention
6. Save versioning rules to Claude memory for persistence across conversations

## Out of scope

- `tools/vscode-extension/` — already deleted
- Benchmarks runner — internal tool, version is informational only
- Automated version bump scripts — future work
- npm publication process fixes — handled in separate change
