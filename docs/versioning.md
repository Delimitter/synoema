# Synoema Versioning Policy

## Current Version

**0.1.0-alpha.2**

## Version Scheme

```
MAJOR.MINOR.PATCH-STAGE.N
```

| Component | Meaning |
|-----------|---------|
| `MAJOR` | Breaking changes to the language or core public API |
| `MINOR` | New features, backwards-compatible |
| `PATCH` | Bug fixes, no new features |
| `STAGE` | `alpha` / `beta` / _(absent = stable)_ |
| `N` | Release counter within the stage (1, 2, 3, …) |

Examples: `0.1.0-alpha.1`, `0.1.0-alpha.2`, `0.2.0-beta.1`, `1.0.0`

## Stages

### Alpha (`0.x.y-alpha.N`)

Current stage.

**Guarantees:**
- The compiler builds and the test suite passes (0 failures, 0 warnings)
- Documented examples work as shown in README

**No guarantees:**
- Language syntax may change between alpha releases
- Cranelift JIT ABI is not stable across versions
- MCP tool/resource API may change
- Standard library functions may be renamed or removed
- `Cargo.toml` crate names and public Rust APIs may change

**Recommended for:** researchers, early adopters, language explorers.

---

### Beta (`0.x.y-beta.N`)

Not yet reached.

**Guarantees (once entered):**
- Language syntax is frozen within a MINOR version
- MCP `tools/list` API is stable (tool names and input schemas)
- JIT ABI is stable within a MINOR version
- Core stdlib functions (`map`, `filter`, `foldl`, `show`, `print`, `readline`) are stable

**No guarantees:**
- Experimental stdlib extensions may still change
- Cranelift codegen internals are not public API

**Recommended for:** prototypes, experiments, early integrations.

---

### Stable (`1.x.y`)

Not yet reached.

**Guarantees:**
- Full semantic versioning (SemVer 2.0)
- Language syntax: backwards-compatible within MAJOR
- All documented MCP APIs: stable within MAJOR
- `synoema-eval` and `synoema-types` public Rust crate APIs: stable within MAJOR
- Migration guides provided for any MAJOR bump

---

## Ecosystem Components

All user-facing components share a single version number. The canonical version lives in `lang/Cargo.toml` → `[workspace.package].version`.

| Component | File | Sync Mechanism |
|-----------|------|----------------|
| Lang crates (8) | `lang/Cargo.toml` | **Source of truth** — workspace inheritance |
| MCP server (Cargo) | `mcp/Cargo.toml` | Manual sync at release |
| MCP server (runtime) | `mcp/synoema-mcp/src/main.rs` | `env!("CARGO_PKG_VERSION")` — automatic |
| MCP npm main | `npm/synoema-mcp/package.json` | CI sets from git tag — automatic |
| MCP npm platforms (4) | `npm/platforms/*/package.json` | CI sets from git tag — automatic |
| VSCode extension | `vscode-extension/package.json` | Manual sync (`X.Y.Z`, no pre-release suffix) |
| Docs / README | `README.md`, `docs/*.md` | Manual sync |
| Release binaries | `releases/vX.Y.Z[-stage.N]/` | Directory named by version |

**Internal tools** (e.g., `benchmarks/runner`) are excluded — they use their own informational versions.

**VSCode Marketplace note:** the marketplace does not support pre-release suffixes in version strings. VSCode extension uses `MAJOR.MINOR.PATCH` (e.g., `0.1.0`) while all other components use the full `MAJOR.MINOR.PATCH-STAGE.N` (e.g., `0.1.0-alpha.1`).

## Release Process

1. **Alpha releases** — tagged `v0.x.y-alpha.N` on every significant feature completion or bug fix batch
2. **Beta entry** — when syntax is stable and all planned 1.0 features are in alpha
3. **Stable 1.0** — after beta period with no breaking changes

## Compatibility Table

| What | Alpha | Beta | Stable |
|------|:-----:|:----:|:------:|
| Language syntax | ✗ | ✓ | ✓ |
| MCP tools API | ✗ | ✓ | ✓ |
| JIT ABI | ✗ | ✓ | ✓ |
| Core stdlib | ✗ | ✓ | ✓ |
| Rust crate API | ✗ | ✗ | ✓ |
| SemVer guarantees | ✗ | ✗ | ✓ |

## Pre-Tag Checklist

Before creating a git tag `vX.Y.Z-stage.N`:

1. `lang/Cargo.toml` → `[workspace.package].version = "X.Y.Z-stage.N"`
2. `mcp/Cargo.toml` → `[workspace.package].version = "X.Y.Z-stage.N"`
3. `vscode-extension/package.json` → `"version": "X.Y.Z"`
4. `README.md` → version badge line
5. `docs/versioning.md` → version history table (add row)
6. `context/PROJECT_STATE.md` → version line
7. `docs/install.md` → version badge
8. `docs/mcp.md` → version badge
9. `cargo test` — 0 failures, 0 warnings
10. `cargo build --release` in `mcp/` — MCP server compiles

## NPM Release Process

Synoema publishes 5 npm packages per release:

| Package | Contents |
|---------|----------|
| `synoema-mcp` | Launcher (bin/run.js) + optionalDependencies |
| `@delimitter/mcp-darwin-arm64` | Native binary — macOS Apple Silicon |
| `@delimitter/mcp-darwin-x64` | Native binary — macOS Intel |
| `@delimitter/mcp-linux-x64` | Native binary — Linux x86_64 |
| `@delimitter/mcp-win32-x64` | Native binary — Windows x64 |

### Automated (CI)

CI injects the version from the git tag into all package.json files before publishing. npm dist-tag is selected automatically:

| Stage | npm tag |
|-------|---------|
| `alpha` | `alpha` |
| `beta` | `beta` |
| stable (no stage) | `latest` |

### Manual Release Checklist

1. Ensure `cargo test` passes (0 failures, 0 warnings)
2. Bump version: `./scripts/npm-bump-version.sh <version>`
3. Commit: `git commit -m "chore: bump npm packages to <version>"`
4. Tag: `git tag v<version>`
5. Push tag: `git push origin v<version>`
6. CI builds binaries, publishes npm packages, creates GitHub Release

### Install

```bash
# npx (recommended, no install)
npx synoema-mcp

# Global install
npm install -g synoema-mcp
synoema-mcp
```

## Version History

| Version | Stage | Date | Notes |
|---------|-------|------|-------|
| 0.1.0-alpha.1 | alpha | 2026-04 | Initial release: full JIT, MCP server, structured diagnostics |
| 0.1.0-alpha.2 | alpha | 2026-04 | JSON stdlib, compiler bugfixes, init improvements |
