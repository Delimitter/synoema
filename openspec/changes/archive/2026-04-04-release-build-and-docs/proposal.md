# Proposal: Release Build Makefiles & Practical Documentation

## Problem Statement

The `releases/v0.1.0-alpha.1/` directory has platform subdirectories with READMEs that describe how to download pre-built binaries from GitHub Releases. But:

1. **No local build mechanism** — a developer who clones the repo cannot build binaries for a given platform from inside the release directory. There's no Makefile or build script.
2. **Documentation is download-centric, not build-centric** — every platform README starts with `curl` commands to download from GitHub, but doesn't explain how to build from source in a practical, copy-paste way.
3. **No usage guide after build** — after building, the user needs to know: where's the binary? How to run it? What commands exist?

## Solution Overview

### 1. Makefile per platform directory

Add a `Makefile` to each `releases/v0.1.0-alpha.1/<platform>/` directory that:
- Builds `synoema` (repl) and `synoema-mcp` from source with the correct Rust target
- Copies binaries into the current directory
- Provides `make`, `make clean`, `make install` targets

### 2. Rewrite platform READMEs

Replace download-first READMEs with practical build-first documentation:
- **Build from source** — step by step, prerequisites, single command
- **Quick start after build** — the 5 commands every user needs
- **MCP setup** — kept but simplified
- Keep download option as alternative, not primary

### 3. Rewrite top-level releases README

Practical orientation: "here's how to build" before "here's how to download."

## Scope

### In Scope
- Makefile for darwin-arm64, darwin-x64, linux-x64, win32-x64
- Rewrite 4 platform READMEs + 1 release README + 1 top-level releases README
- Build from source instructions with prerequisites

### Out of Scope
- CI/CD changes (release.yml is fine)
- New binaries in git (binaries are built locally or via CI)
- Cross-compilation setup guides

## Success Criteria

1. `cd releases/v0.1.0-alpha.1/darwin-arm64 && make` builds both binaries
2. Each platform README has copy-paste build instructions
3. Each README has a "Quick Start" section with 5 essential commands
4. `make install` copies binaries to `/usr/local/bin/` (Unix) or prints instructions (Windows)
