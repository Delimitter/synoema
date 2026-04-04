# Design: npm-publication

## Decision 1: Version Synchronization

**Approach:** Reset all 5 package.json to `0.1.0-alpha.1`. CI workflow injects version from git tag via `npm version` before publish.

**Why not keep alpha.4:** Пакет никогда не публиковался, alpha.4 — фантомный bump. Reset к alpha.1 соответствует versioning.md history table.

## Decision 2: Version Bump Script

**File:** `scripts/npm-bump-version.sh`

**Logic:**
```
Usage: ./scripts/npm-bump-version.sh <version>
Example: ./scripts/npm-bump-version.sh 0.1.0-alpha.2
```

1. Validate version format (`MAJOR.MINOR.PATCH[-STAGE.N]`)
2. Update `version` in all 5 package.json
3. Update `optionalDependencies` versions in main package
4. Print summary

**Implementation:** `sed` — zero dependencies, works in CI and locally.

## Decision 3: CI Version Injection

**Modify:** `.github/workflows/release.yml`

Add step before each `npm publish`:
```yaml
- name: Set npm package version
  run: npm version $VERSION --no-git-tag-version --allow-same-version
```

This overwrites static version in package.json with version from git tag. Ensures published version always matches release tag.

## Decision 4: Documentation Updates

| File | Change |
|------|--------|
| `docs/versioning.md` | Add "NPM Release Process" section with checklist |
| `releases/README.md` | Add npm install instructions |
| `context/PROJECT_STATE.md` | Note npm distribution in quick status |

## Files Changed

- `npm/synoema-mcp/package.json` — version reset
- `npm/platforms/*/package.json` (4 files) — version reset
- `.github/workflows/release.yml` — version injection steps
- `scripts/npm-bump-version.sh` — new file
- `docs/versioning.md` — npm release process
- `releases/README.md` — npm install section
- `context/PROJECT_STATE.md` — distribution note
