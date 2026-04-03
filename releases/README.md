# Synoema — Pre-built Binaries

Pre-built binaries for the Synoema compiler and MCP server.

> See [versioning policy](../docs/versioning.md) for version guarantees.

## Latest Release

**[v0.1.0-alpha.1](v0.1.0-alpha.1/)** — April 2026

## Binaries per release

Each release contains two binaries:

| Binary | Description |
|--------|-------------|
| `synoema` | CLI: `run` (interpreter), `jit` (Cranelift), `eval`, REPL |
| `synoema-mcp` | MCP server for Claude Desktop / Cursor / Zed |

## Platforms

| Directory | Platform |
|-----------|---------|
| `darwin-arm64/` | macOS Apple Silicon (M1/M2/M3/M4) |
| `darwin-x64/` | macOS Intel (x86_64) |
| `linux-x64/` | Linux x86_64 |
| `win32-x64/` | Windows x64 |

## How to download

The binary files themselves are hosted as GitHub Release assets (not committed to git).
Download from the [Releases page](https://github.com/Delimitter/synoema/releases) or follow the platform-specific instructions in each directory's README.

Full user guide: [docs/install.md](../docs/install.md)

## Build from source

If no pre-built binary is available for your platform:

```bash
git clone https://github.com/Delimitter/synoema
cd synoema/lang && cargo build --release   # → lang/target/release/synoema
cd synoema/mcp  && cargo build --release   # → mcp/target/release/synoema-mcp
```
