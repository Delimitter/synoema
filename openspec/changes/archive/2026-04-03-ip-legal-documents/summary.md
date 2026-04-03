# IP & Legal Documents

## What was done

Created complete IP/legal infrastructure for the Synoema project:

### License files (6)
- `LICENSE` (root, Apache-2.0) — already existed, verified
- `spec/LICENSE` (CC-BY-SA-4.0)
- `lang/crates/synoema-codegen/LICENSE` (BSL-1.1)
- `tools/LICENSE` (BSL-1.1)
- `docs/LICENSE` (CC-BY-SA-4.0)
- `examples/LICENSE` (MIT-0)

### Governance & policy (6)
- `NOTICE` — attributions
- `TRADEMARK.md` — trademark usage policy
- `CONTRIBUTING.md` — DCO, code standards, license headers
- `CODE_OF_CONDUCT.md` — Contributor Covenant 2.1
- `SECURITY.md` — vulnerability disclosure policy
- `PATENTS` — defensive patent pledge

### Strategy & templates (4)
- `docs/IP_STRATEGY.md` — IP strategy reference
- `.github/ISSUE_TEMPLATE/bug_report.md`
- `.github/ISSUE_TEMPLATE/feature_request.md`
- `.github/PULL_REQUEST_TEMPLATE.md`

### Automation (2)
- `scripts/add_headers.sh` — SPDX header tool (idempotent)
- `.githooks/pre-commit` — blocks commits without SPDX headers

### Rules update
- `context/RULES.md` section 8 — licensing & IP rules

### Headers applied
- 52 source files received SPDX headers (34 Apache-2.0, 4 BUSL-1.1, 14 MIT-0)

## Verification
- All 702 tests pass after changes
- Pre-commit hook tested and working
- add_headers.sh idempotent (re-run: 0 changed, 52 skipped)
