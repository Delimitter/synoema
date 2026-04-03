# Synoema MCP Server

The Synoema MCP server integrates the Synoema compiler and evaluator into any MCP-compatible LLM toolchain — Claude Desktop, Cursor, Zed, and any client implementing the [Model Context Protocol](https://modelcontextprotocol.io).

> **Version:** 0.1.0-alpha.1 — see [versioning policy](versioning.md)

## What It Provides

### Tools

| Tool | Input | Output |
|------|-------|--------|
| `eval` | Synoema expression (e.g. `6 * 7`) | Value + inferred type, or structured error |
| `typecheck` | Full Synoema program (with `main`) | `main : Type` or structured errors |
| `run` | Full Synoema program (with `main`) | stdout output + final value |

### Resources

| URI | Description |
|-----|-------------|
| `synoema://spec/language_reference` | Full language specification |
| `synoema://spec/llm_ref` | Minified reference optimized for LLM generation (≤1500 tokens) |
| `synoema://examples` | Index of all example programs |
| `synoema://examples/{name.sno}` | Source of a specific example |

### Prompts

| Prompt | Description |
|--------|-------------|
| `synoema_codegen` | System prompt for Synoema code generation — syntax rules, common mistakes, examples |

## Quick Start — npx (Recommended)

No installation required. Run directly with Node.js ≥16:

```bash
npx synoema-mcp
```

npm automatically downloads the binary for your platform on first run.

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

**Or install globally:**
```bash
npm install -g synoema-mcp
# then use: synoema-mcp (no npx needed)
```

## Quick Start — Pre-built Binary

Download directly without npm. Available from the [releases directory](../releases/):

```bash
# macOS (Apple Silicon)
curl -L https://github.com/Delimitter/synoema/releases/download/v0.1.0-alpha.1/synoema-mcp-0.1.0-alpha.1-darwin-arm64 \
  -o synoema-mcp && chmod +x synoema-mcp

# macOS (Intel)
curl -L https://github.com/Delimitter/synoema/releases/download/v0.1.0-alpha.1/synoema-mcp-0.1.0-alpha.1-darwin-x64 \
  -o synoema-mcp && chmod +x synoema-mcp

# Linux (x86_64)
curl -L https://github.com/Delimitter/synoema/releases/download/v0.1.0-alpha.1/synoema-mcp-0.1.0-alpha.1-linux-x64 \
  -o synoema-mcp && chmod +x synoema-mcp
```

Full instructions: [docs/install.md](install.md)

## Build from Source

Requires: Rust toolchain (stable, ≥1.75).

```bash
git clone https://github.com/Delimitter/synoema
cd synoema/mcp
cargo build --release
# Binary: target/release/synoema-mcp
```

The MCP server has no external runtime dependencies — it embeds the Synoema evaluator and type checker.

> **Note:** `synoema-mcp` depends on crates in `lang/` via relative paths. Always build from `mcp/` inside the cloned repository.

## Install via cargo

```bash
# From repo root
cargo install --path mcp/synoema-mcp

# Verify
synoema-mcp --help
```

## Connect to Claude Desktop

Edit `claude_desktop_config.json`:

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

### Using a pre-built binary

```json
{
  "mcpServers": {
    "synoema": {
      "command": "/path/to/synoema-mcp"
    }
  }
}
```

### Using cargo-installed binary

```json
{
  "mcpServers": {
    "synoema": {
      "command": "synoema-mcp"
    }
  }
}
```

### With SYNOEMA_ROOT (if examples don't load)

If the server can't find example files, set the repo root explicitly:

```json
{
  "mcpServers": {
    "synoema": {
      "command": "/path/to/synoema-mcp",
      "env": {
        "SYNOEMA_ROOT": "/path/to/synoema"
      }
    }
  }
}
```

After editing, restart Claude Desktop.

## Connect to Other MCP Clients

The server communicates over **stdio** (JSON-RPC 2.0, MCP 2024-11-05). Any MCP-compatible client works.

**Cursor** — add to `.cursor/mcp.json`:
```json
{
  "synoema": { "command": "/path/to/synoema-mcp" }
}
```

**Zed** — add to `settings.json` under `context_servers`:
```json
{
  "context_servers": {
    "synoema": { "command": { "path": "/path/to/synoema-mcp", "args": [] } }
  }
}
```

**Manual / testing:**
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0"}}}' | ./synoema-mcp
```

## Usage Examples

Once connected to Claude Desktop, you can ask Claude:

> *"Use the eval tool to compute fib(20) in Synoema"*

> *"Typecheck this Synoema program: `main = map (\x -> x * 2) [1 2 3]`"*

> *"Read the Synoema language reference resource and write a quicksort"*

> *"Use the synoema_codegen prompt before generating code"*

## Tool Examples

**eval:**
```
Input:  { "code": "[1..10] |> filter (\\x -> x % 2 == 0) |> sum" }
Output: 30 : Int
```

**typecheck:**
```
Input:  { "code": "main = map (\\x -> x * 2) [1 2 3]" }
Output: main : List Int
```

**run:**
```
Input:  { "code": "main = print \"Hello, Synoema!\"" }
Output: Hello, Synoema!
        ()
```

## Troubleshooting

**Server doesn't start:** Check that the binary has execute permission (`chmod +x`). On macOS, if you see "cannot be opened because the developer cannot be verified", run:
```bash
xattr -dr com.apple.quarantine synoema-mcp
```

**Examples not found:** Set `SYNOEMA_ROOT` environment variable to the repo root (see config above).

**Type errors in eval:** `eval` runs single expressions. For multi-binding programs, use `run` with a `main` binding.

**Protocol version mismatch:** The server implements MCP 2024-11-05. Ensure your client supports this version.
