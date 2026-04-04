# Design: Release Build Makefiles & Documentation

## Makefile Design

### Shared pattern, platform-specific variables

All 4 Makefiles follow the same structure. Only `TARGET` and `EXT` differ.

```makefile
TARGET = <rust-target-triple>
EXT    = <empty or .exe>
# ... rest is identical
```

### Path strategy

Makefiles live in `releases/v0.1.0-alpha.1/<platform>/`. They reference the source via relative paths:

```
releases/v0.1.0-alpha.1/darwin-arm64/Makefile
  → ../../../lang/    (LANG_DIR)
  → ../../../mcp/     (MCP_DIR)
```

### Native vs cross-compilation

The Makefile targets the platform it's in. On a matching host, `make` just works. On a non-matching host, the user needs the Rust target installed (`rustup target add <target>`). The Makefile does NOT auto-install targets — it prints an error message if the target is missing.

### Windows: no make

Windows directory gets a `build.ps1` PowerShell script instead of a Makefile. Same targets as make: build, clean, install.

## Documentation Design

### README rewrite principle

**Build-first, download-second.** Current READMEs lead with download URLs. New READMEs lead with "clone & make."

### Structure per platform README

```
# Synoema v0.1.0-alpha.1 — <Platform Name>

## Prerequisites
## Build
## Quick Start
## Install (optional)
## MCP Setup (optional)
## Alternative: Download Pre-built Binary
```

### Quick Start — 5 commands

Every platform README includes the same 5 commands:
1. `synoema eval "6 * 7"` — evaluate expression
2. `synoema run file.sno` — run file (interpreter)
3. `synoema jit file.sno` — run file (JIT, faster)
4. `synoema` — interactive REPL
5. `synoema test examples/` — run tests

### Top-level README

Simplified: what + where + how. Table of platforms with one-liner build command each.
