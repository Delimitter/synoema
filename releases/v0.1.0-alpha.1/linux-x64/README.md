# Synoema v0.1.0-alpha.1 — Linux x86_64

**Platform:** Linux x86_64 (glibc 2.17+: Ubuntu 18.04+, Debian 9+, Fedora 28+)

## Prerequisites

- [Rust toolchain](https://rustup.rs/) (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Target: `rustup target add x86_64-unknown-linux-gnu` (usually pre-installed)

## Build from source

```bash
cd releases/v0.1.0-alpha.1/linux-x64
make
```

This produces two binaries in the current directory:

| Binary | What it does |
|--------|-------------|
| `synoema` | Compiler CLI: eval, run, jit, repl, test, build |
| `synoema-mcp` | MCP server for Claude Desktop / Cursor / Zed |

## Quick Start

```bash
# Evaluate an expression
./synoema eval "6 * 7"
# → 42

# Run a file (interpreter)
./synoema run ../../../lang/examples/quicksort.sno
# → [1, 2, 3, 4, 5, 6, 7, 8, 9]

# Run with JIT (faster)
./synoema jit ../../../lang/examples/factorial.sno
# → 3628800

# Interactive REPL
./synoema

# Run doctests in a directory
./synoema test ../../../lang/examples/
```

## Install (optional)

```bash
sudo make install
# Copies synoema and synoema-mcp to /usr/local/bin/
```

After install, use `synoema` from anywhere without `./`.

## MCP Setup (Claude Desktop)

Edit `~/.config/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "synoema": {
      "command": "/usr/local/bin/synoema-mcp"
    }
  }
}
```

Restart Claude Desktop. Tools `eval`, `typecheck`, `run` become available.

## Static build (musl)

If you need a fully static binary (no glibc dependency):

```bash
rustup target add x86_64-unknown-linux-musl
# Edit Makefile: change TARGET to x86_64-unknown-linux-musl
make
```

## Alternative: Download Pre-built Binary

```bash
curl -L https://github.com/Delimitter/synoema/releases/download/v0.1.0-alpha.1/synoema-0.1.0-alpha.1-linux-x64 -o synoema
curl -L https://github.com/Delimitter/synoema/releases/download/v0.1.0-alpha.1/synoema-mcp-0.1.0-alpha.1-linux-x64 -o synoema-mcp
chmod +x synoema synoema-mcp
```
