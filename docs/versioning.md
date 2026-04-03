# Synoema Versioning Policy

## Current Version

**0.1.0-alpha.1**

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

## Version History

| Version | Stage | Date | Notes |
|---------|-------|------|-------|
| 0.1.0-alpha.1 | alpha | 2026-04 | Initial release: full JIT, MCP server, structured diagnostics |
