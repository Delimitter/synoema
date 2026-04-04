# Spec: Ecosystem Versioning

## Single Source of Truth

The canonical version lives in `lang/Cargo.toml` → `[workspace.package].version`.

All other version references MUST be derived from this value — either at build time (env macro, CI extraction) or kept manually in sync during release.

## Version Components

| Component | File | Mechanism |
|-----------|------|-----------|
| Lang crates (8) | `lang/Cargo.toml` workspace | `version.workspace = true` (already works) |
| MCP Cargo workspace | `mcp/Cargo.toml` | Manual sync with lang workspace |
| MCP binary (runtime) | `mcp/synoema-mcp/src/main.rs` | `env!("CARGO_PKG_VERSION")` macro |
| MCP npm main | `npm/synoema-mcp/package.json` | CI sets from git tag (already works) |
| MCP npm platforms | `npm/platforms/*/package.json` | CI sets from git tag (already works) |
| VSCode extension | `vscode-extension/package.json` | Manual sync |
| Docs/README | `README.md`, `docs/*.md`, `context/PROJECT_STATE.md` | Manual sync |
| Release directory | `releases/v{VERSION}/` | Named by version |

## Rules

1. **All user-facing components** share the same `MAJOR.MINOR.PATCH-STAGE.N` version
2. **Internal tools** (benchmarks) may use their own version, not bound by this policy
3. **At release time**, all files in the table above MUST show the same version
4. **Hardcoded version strings** are forbidden — use `env!("CARGO_PKG_VERSION")` in Rust, CI-derived values elsewhere
5. **npm package.json versions** in the repo are starting points; CI overrides them from git tag (existing behavior, correct)
6. **Pre-release tags** (`-alpha.N`, `-beta.N`) are mandatory until stable 1.0

## OpenSpec Change Convention

When an OpenSpec change is created, the change directory name MAY include a version reference if the change targets a specific release:

```
openspec/changes/<change-name>/         # no version — applies to current
openspec/changes/v0.2.0/<change-name>/  # version-scoped changes (future)
```

Current convention: flat `openspec/changes/<name>/`. Version-scoped directories are a future consideration.

## Release Checklist

Before tagging `vX.Y.Z-stage.N`:

- [ ] `lang/Cargo.toml` → `[workspace.package].version = "X.Y.Z-stage.N"`
- [ ] `mcp/Cargo.toml` → `[workspace.package].version = "X.Y.Z-stage.N"`
- [ ] `vscode-extension/package.json` → `"version": "X.Y.Z"` (npm semver, no pre-release for marketplace)
- [ ] `README.md` version badge
- [ ] `docs/versioning.md` → version history table
- [ ] `context/PROJECT_STATE.md` version line
- [ ] `docs/install.md` version badge
- [ ] `docs/mcp.md` version badge
- [ ] `CONTRIBUTING.md` release path references
