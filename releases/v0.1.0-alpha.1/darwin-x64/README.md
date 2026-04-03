# Synoema v0.1.0-alpha.1 — macOS Intel (darwin-x64)

**Platform:** macOS 12+ (Monterey or later), Intel x86_64

## Download

```bash
curl -L https://github.com/Delimitter/synoema/releases/download/v0.1.0-alpha.1/synoema-0.1.0-alpha.1-darwin-x64 \
  -o synoema

curl -L https://github.com/Delimitter/synoema/releases/download/v0.1.0-alpha.1/synoema-mcp-0.1.0-alpha.1-darwin-x64 \
  -o synoema-mcp
```

## Setup

```bash
# Make executable
chmod +x synoema synoema-mcp

# Remove macOS quarantine flag (required for unsigned binaries)
xattr -dr com.apple.quarantine synoema synoema-mcp

# Optional: move to PATH
sudo mv synoema /usr/local/bin/
sudo mv synoema-mcp /usr/local/bin/
```

## Run

```bash
# Evaluate an expression
synoema eval "6 * 7"
# → 42

# Run a file (interpreter)
synoema run examples/quicksort.sno
# → [1 2 3 4 5 6 7 8 9]

# JIT compile and run (4.4× faster)
synoema jit examples/factorial.sno
# → 3628800

# Interactive REPL
synoema
```

## MCP Server (Claude Desktop)

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "synoema": {
      "command": "/usr/local/bin/synoema-mcp"
    }
  }
}
```

Restart Claude Desktop. The tools `eval`, `typecheck`, `run` will be available.

Full MCP documentation: [docs/mcp.md](../../../docs/mcp.md)
