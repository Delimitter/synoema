# Installing Synoema

This guide covers all installation methods for the Synoema compiler (`synoema`) and MCP server (`synoema-mcp`).

> **Version:** 0.1.0-alpha.2 — see [versioning policy](versioning.md)

## Option 0: npx (MCP server only, easiest)

Requires Node.js ≥16. No Rust, no manual download.

```bash
npx synoema-mcp
```

npm downloads the binary for your platform automatically on first run.

**Claude Desktop config:**
```json
{
  "mcpServers": {
    "synoema": {
      "command": "npx",
      "args": ["synoema-mcp"]
    }
  }
}
```

**Global install:**
```bash
npm install -g synoema-mcp
synoema-mcp   # no npx needed
```

---

## Option 1: Pre-built Binary (Recommended)

No Rust toolchain required. Download the binary for your platform.

### Step 1 — Determine your platform

| Platform | Directory |
|----------|-----------|
| macOS Apple Silicon (M1/M2/M3/M4) | `darwin-arm64` |
| macOS Intel | `darwin-x64` |
| Linux x86_64 | `linux-x64` |
| Windows x64 | `win32-x64` |

### Step 2 — Download

Download from [GitHub Releases](https://github.com/Delimitter/synoema/releases/latest) or see the platform-specific README:

- [macOS Apple Silicon](../releases/v0.1.0-alpha.2/darwin-arm64/)
- [macOS Intel](../releases/v0.1.0-alpha.2/darwin-x64/)
- [Linux x86_64](../releases/v0.1.0-alpha.2/linux-x64/)
- [Windows x64](../releases/v0.1.0-alpha.2/win32-x64/)

### Step 3 — Install

**Automatic (recommended):**
```bash
chmod +x synoema              # macOS/Linux only
xattr -dr com.apple.quarantine synoema synoema-mcp  # macOS only

./synoema install
# → copies to ~/.synoema/bin/
# → adds to PATH (zsh/bash/fish on Unix, User PATH on Windows)
```

Options:
- `--prefix <path>` — install to `<path>/bin/` instead of `~/.synoema/bin/`
- `--no-path` — don't modify shell profile / PATH

If `synoema-mcp` is in the same directory, it gets installed automatically.

**Manual alternative:**

macOS / Linux:
```bash
chmod +x synoema synoema-mcp
xattr -dr com.apple.quarantine synoema synoema-mcp  # macOS only
sudo mv synoema /usr/local/bin/
sudo mv synoema-mcp /usr/local/bin/
```

Windows (PowerShell):
```powershell
Move-Item synoema.exe C:\Tools\synoema.exe
Move-Item synoema-mcp.exe C:\Tools\synoema-mcp.exe
```

### Step 4 — Verify

```bash
synoema eval "6 * 7"
# → 42
```

---

## Option 2: cargo install

Requires: Rust stable ≥1.75 ([rustup.rs](https://rustup.rs)).

```bash
git clone https://github.com/Delimitter/synoema
cd synoema

# Install the compiler/repl
cargo install --path lang/crates/synoema-repl

# Install the MCP server
cargo install --path mcp/synoema-mcp
```

Binaries are placed in `~/.cargo/bin/` (usually on PATH).

---

## Option 3: Build from Source

```bash
git clone https://github.com/Delimitter/synoema
cd synoema

# Build compiler
cd lang && cargo build --release
# → lang/target/release/synoema

# Build MCP server
cd ../mcp && cargo build --release
# → mcp/target/release/synoema-mcp
```

---

## Running the Compiler

```bash
# Interpreter — all language features
synoema run examples/quicksort.sno

# JIT — Cranelift native code (4.4× faster)
synoema jit examples/factorial.sno

# Evaluate a single expression
synoema eval "[1..10] |> filter (\x -> x % 2 == 0) |> sum"

# Interactive REPL
synoema

# Structured JSON errors (for LLM toolchains)
synoema --errors json run file.sno
```

---

## Running the MCP Server

The MCP server communicates over stdio (JSON-RPC 2.0). Configure your MCP client to launch it as a subprocess.

### Claude Desktop

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "synoema": {
      "command": "/usr/local/bin/synoema-mcp"
    }
  }
}
```

If examples are not found, add `SYNOEMA_ROOT`:

```json
{
  "mcpServers": {
    "synoema": {
      "command": "/usr/local/bin/synoema-mcp",
      "env": {
        "SYNOEMA_ROOT": "/path/to/synoema"
      }
    }
  }
}
```

Restart Claude Desktop after editing.

Full MCP documentation: [docs/mcp.md](mcp.md)

---

## Troubleshooting

**`synoema: command not found`** — ensure the binary directory is in your `PATH`.

**macOS: "cannot be opened because the developer cannot be verified"**:
```bash
xattr -dr com.apple.quarantine synoema synoema-mcp
```

**`error: could not find synoema-diagnostic`** when building from source — ensure you're building from the repo root with `cargo build` (not from a crate subdirectory alone).

**MCP server: examples not found** — set `SYNOEMA_ROOT` to the repo root (see above).

**Linux: `/lib/x86_64-linux-gnu/libc.so.6: version 'GLIBC_2.xx' not found`** — your glibc is too old. Build from source with musl target:
```bash
rustup target add x86_64-unknown-linux-musl
cd mcp && cargo build --release --target x86_64-unknown-linux-musl
```
