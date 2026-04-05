# Synoema v0.1.0-alpha.1 — Windows x64

**Platform:** Windows 10/11 x64

## Prerequisites

- [Rust toolchain](https://www.rust-lang.org/tools/install) (download `rustup-init.exe`)
- Visual Studio Build Tools (C++ workload) or Visual Studio with C++ support
- Target: `rustup target add x86_64-pc-windows-msvc` (usually pre-installed)

## Build from source

```powershell
cd releases\v0.1.0-alpha.1\win32-x64
.\build.ps1
```

This produces two binaries in the current directory:

| Binary | What it does |
|--------|-------------|
| `synoema.exe` | Compiler CLI: eval, run, jit, repl, test, build |
| `synoema-mcp.exe` | MCP server for Claude Desktop / Cursor / Zed |

## Quick Start

```powershell
# Evaluate an expression
.\synoema.exe eval "6 * 7"
# → 42

# Run a file (interpreter)
.\synoema.exe run ..\..\..\lang\examples\quicksort.sno
# → [1, 2, 3, 4, 5, 6, 7, 8, 9]

# Run with JIT (faster)
.\synoema.exe jit ..\..\..\lang\examples\factorial.sno
# → 3628800

# Interactive REPL
.\synoema.exe

# Run doctests in a directory
.\synoema.exe test ..\..\..\lang\examples\
```

## Install (optional)

```powershell
.\build.ps1 -Install
# Copies to C:\Tools\ and prints PATH instructions
```

After install, use `synoema` from anywhere without `.\`.

## MCP Setup (Claude Desktop)

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

Restart Claude Desktop. Tools `eval`, `typecheck`, `run` become available.

## Windows Defender

If Defender flags the binary, add an exclusion for the binary path or compile from source (which you just did).

## Alternative: Download Pre-built Binary

```powershell
Invoke-WebRequest -Uri "https://github.com/Delimitter/synoema/releases/download/v0.1.0-alpha.1/synoema-0.1.0-alpha.1-win32-x64.exe" -OutFile "synoema.exe"
Invoke-WebRequest -Uri "https://github.com/Delimitter/synoema/releases/download/v0.1.0-alpha.1/synoema-mcp-0.1.0-alpha.1-win32-x64.exe" -OutFile "synoema-mcp.exe"
```
