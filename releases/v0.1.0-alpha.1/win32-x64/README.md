# Synoema v0.1.0-alpha.1 — Windows x64

**Platform:** Windows 10/11 x64

## Download

Download from [GitHub Releases](https://github.com/Delimitter/synoema/releases/tag/v0.1.0-alpha.1):

- `synoema-0.1.0-alpha.1-win32-x64.exe`
- `synoema-mcp-0.1.0-alpha.1-win32-x64.exe`

Or via PowerShell:

```powershell
Invoke-WebRequest `
  -Uri "https://github.com/Delimitter/synoema/releases/download/v0.1.0-alpha.1/synoema-0.1.0-alpha.1-win32-x64.exe" `
  -OutFile "synoema.exe"

Invoke-WebRequest `
  -Uri "https://github.com/Delimitter/synoema/releases/download/v0.1.0-alpha.1/synoema-mcp-0.1.0-alpha.1-win32-x64.exe" `
  -OutFile "synoema-mcp.exe"
```

## Setup

Move the executables to a directory in your `PATH`, e.g. `C:\Tools\`:

```powershell
Move-Item synoema.exe C:\Tools\synoema.exe
Move-Item synoema-mcp.exe C:\Tools\synoema-mcp.exe
```

Add `C:\Tools` to your system PATH if not already there.

## Run

```powershell
# Evaluate an expression
synoema eval "6 * 7"
# → 42

# Run a file (interpreter)
synoema run examples\quicksort.sno
# → [1 2 3 4 5 6 7 8 9]

# JIT compile and run
synoema jit examples\factorial.sno
# → 3628800

# Interactive REPL
synoema
```

## MCP Server (Claude Desktop)

Edit `%APPDATA%\Claude\claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "synoema": {
      "command": "C:\\Tools\\synoema-mcp.exe"
    }
  }
}
```

Restart Claude Desktop. The tools `eval`, `typecheck`, `run` will be available.

Full MCP documentation: [docs/mcp.md](../../../docs/mcp.md)

## Windows Defender

If Windows Defender flags the binary, you may need to add an exclusion for the binary path, or compile from source:

```powershell
rustup target add x86_64-pc-windows-msvc
cd mcp
cargo build --release
```
