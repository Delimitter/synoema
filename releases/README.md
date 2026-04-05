# Synoema — Releases

Pre-built binaries and build scripts for the Synoema compiler and MCP server.

## Latest Release

**[v0.1.0-alpha.2](v0.1.0-alpha.2/)** — April 2026

## How to build

```bash
# 1. Clone the repo (if you haven't)
git clone https://github.com/Delimitter/synoema
cd synoema

# 2. Go to your platform's release directory
cd releases/v0.1.0-alpha.2/darwin-arm64   # macOS Apple Silicon
# cd releases/v0.1.0-alpha.2/darwin-x64   # macOS Intel
# cd releases/v0.1.0-alpha.2/linux-x64    # Linux x86_64
# cd releases/v0.1.0-alpha.2/win32-x64    # Windows (use .\build.ps1)

# 3. Build
make

# 4. Run
./synoema eval "6 * 7"   # → 42
```

**Prerequisite:** [Rust toolchain](https://rustup.rs/).

## What gets built

| Binary | Description |
|--------|-------------|
| `synoema` | CLI: `run` (interpreter), `jit` (Cranelift), `eval`, `build`, `test`, REPL |
| `synoema-mcp` | MCP server for Claude Desktop / Cursor / Zed |

## Platforms

| Directory | Platform | Build |
|-----------|---------|-------|
| `darwin-arm64/` | macOS Apple Silicon (M1-M4) | `make` |
| `darwin-x64/` | macOS Intel (x86_64) | `make` |
| `linux-x64/` | Linux x86_64 | `make` |
| `win32-x64/` | Windows x64 | `.\build.ps1` |

## Install via npm

```bash
# MCP server (recommended for Claude Desktop / Cursor / Zed)
npx synoema-mcp

# Or install globally
npm install -g synoema-mcp
synoema-mcp
```

npm handles platform detection automatically — the correct native binary is installed.

## Alternative: Download pre-built binaries

Download from the [GitHub Releases page](https://github.com/Delimitter/synoema/releases).

## Build from workspace root

If you prefer building without the release scripts:

```bash
cd lang && cargo build --release        # → lang/target/release/synoema
cd mcp && cargo build --release         # → mcp/target/release/synoema-mcp
```

Full documentation: [README.md](../README.md)
