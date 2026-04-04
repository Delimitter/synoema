# Spec: Release Build System

## Makefile Requirements

Each platform directory (`darwin-arm64`, `darwin-x64`, `linux-x64`, `win32-x64`) gets a Makefile.

### Targets

| Target | Action |
|--------|--------|
| `all` (default) | Build `synoema` + `synoema-mcp` in release mode, copy to current dir |
| `clean` | Remove built binaries from current dir |
| `install` | Copy binaries to `/usr/local/bin/` (Unix only) |

### Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `LANG_DIR` | Path to `lang/` workspace | `../../../lang` |
| `MCP_DIR` | Path to `mcp/` workspace | `../../../mcp` |
| `TARGET` | Rust target triple | Platform-specific |
| `PREFIX` | Install prefix | `/usr/local` |

### Platform-specific targets

| Platform | TARGET | Binary extension |
|----------|--------|-----------------|
| darwin-arm64 | `aarch64-apple-darwin` | (none) |
| darwin-x64 | `x86_64-apple-darwin` | (none) |
| linux-x64 | `x86_64-unknown-linux-gnu` | (none) |
| win32-x64 | `x86_64-pc-windows-msvc` | `.exe` |

### Build command pattern

```makefile
cargo build --release --target $(TARGET) -p synoema-repl --manifest-path $(LANG_DIR)/Cargo.toml
cargo build --release --target $(TARGET) --manifest-path $(MCP_DIR)/Cargo.toml
```

## Documentation Requirements

### Platform README structure

1. **Title** — platform name, version
2. **Prerequisites** — what needs to be installed (Rust toolchain, target)
3. **Build from source** — `make` command, what it produces
4. **Quick Start** — 5 essential commands after build
5. **Install** — `make install` or manual steps
6. **MCP Setup** — Claude Desktop config (brief)
7. **Alternative: Download** — curl/PowerShell (moved to bottom)

### Top-level releases/README.md

1. Overview of what Synoema is (1 sentence)
2. Platform table with build commands
3. Build from source (generic)
4. Link to full docs
