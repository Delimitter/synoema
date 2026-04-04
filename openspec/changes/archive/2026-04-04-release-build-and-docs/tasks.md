# Tasks: Release Build Makefiles & Documentation

## Makefiles

- [x] Create `releases/v0.1.0-alpha.1/darwin-arm64/Makefile`
- [x] Create `releases/v0.1.0-alpha.1/darwin-x64/Makefile`
- [x] Create `releases/v0.1.0-alpha.1/linux-x64/Makefile`
- [x] Create `releases/v0.1.0-alpha.1/win32-x64/build.ps1` (PowerShell)

## Documentation

- [x] Rewrite `releases/v0.1.0-alpha.1/darwin-arm64/README.md` — build-first, quick start
- [x] Rewrite `releases/v0.1.0-alpha.1/darwin-x64/README.md` — build-first, quick start
- [x] Rewrite `releases/v0.1.0-alpha.1/linux-x64/README.md` — build-first, quick start
- [x] Rewrite `releases/v0.1.0-alpha.1/win32-x64/README.md` — build-first, quick start
- [x] Rewrite `releases/v0.1.0-alpha.1/README.md` — practical overview
- [x] Rewrite `releases/README.md` — top-level, build-oriented

## Validation

- [x] Run `make` in darwin-arm64 to verify build works — builds synoema (3.8M) + synoema-mcp (1.6M)
- [x] Verify binaries are correct Mach-O arm64 executables
- [N/A] `synoema eval` has pre-existing prelude parse error (line 67, nested guard syntax) — not related to this change
