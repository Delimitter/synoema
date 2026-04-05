---
status: active
last_verified: 2026-04-04
---

# MCP Auto-Deploy

## ADDED Requirements

### Requirement: MCP server --version flag
The MCP binary SHALL support `--version` flag that prints version and exits.

#### Scenario: Version query
- **WHEN** user runs `synoema-mcp --version`
- **THEN** stdout prints `synoema-mcp X.Y.Z` and process exits with code 0
- **AND** no JSON-RPC handshake occurs

### Requirement: MCP server --health flag
The MCP binary SHALL support `--health` flag that prints JSON health info and exits.

#### Scenario: Health check
- **WHEN** user runs `synoema-mcp --health`
- **THEN** stdout prints JSON: `{"status":"ok","version":"X.Y.Z","protocol":"2024-11-05"}`
- **AND** process exits with code 0

### Requirement: mcp-install command
The REPL binary SHALL support `synoema mcp-install` command that installs the MCP binary for the current platform.

#### Scenario: Auto-detect and install on macOS ARM
- **WHEN** user runs `synoema mcp-install` on darwin-arm64
- **THEN** the tool downloads `synoema-mcp-{version}-darwin-arm64` from GitHub Releases
- **AND** installs to `~/.synoema/bin/synoema-mcp`
- **AND** prints installed path and version

#### Scenario: Auto-detect and install on Linux x64
- **WHEN** user runs `synoema mcp-install` on linux-x64
- **THEN** the tool downloads `synoema-mcp-{version}-linux-x64` from GitHub Releases
- **AND** installs to `~/.synoema/bin/synoema-mcp`

#### Scenario: Auto-detect and install on Windows x64
- **WHEN** user runs `synoema mcp-install` on win32-x64
- **THEN** the tool downloads `synoema-mcp-{version}-win32-x64.exe` from GitHub Releases
- **AND** installs to `%USERPROFILE%\.synoema\bin\synoema-mcp.exe`

#### Scenario: Custom install path
- **WHEN** user runs `synoema mcp-install --prefix /usr/local`
- **THEN** the binary is installed to `/usr/local/bin/synoema-mcp`

#### Scenario: Build from source fallback
- **WHEN** user runs `synoema mcp-install --from-source`
- **THEN** the tool runs `cargo build --release` in the MCP workspace
- **AND** copies the built binary to the install path

#### Scenario: Unsupported platform
- **WHEN** user runs `synoema mcp-install` on unsupported platform
- **THEN** error message suggests `--from-source` option

### Requirement: mcp-update command
The REPL binary SHALL support `synoema mcp-update` command.

#### Scenario: Check for updates
- **WHEN** user runs `synoema mcp-update`
- **THEN** the tool queries GitHub Releases API for latest version
- **AND** compares with installed version (via `synoema-mcp --version`)
- **AND** if newer version exists, downloads and replaces

#### Scenario: Already up to date
- **WHEN** user runs `synoema mcp-update` and installed version matches latest
- **THEN** prints "synoema-mcp is up to date (X.Y.Z)"

#### Scenario: No MCP binary found
- **WHEN** user runs `synoema mcp-update` but no synoema-mcp is installed
- **THEN** suggests running `synoema mcp-install` first

### Requirement: Version pinning in init
The `synoema init` command SHALL pin MCP version in generated configs.

#### Scenario: Init with npx pinning
- **WHEN** user runs `synoema init myapp --ai claude`
- **THEN** generated `.claude/settings.json` contains `"args": ["synoema-mcp@0.1.0-alpha.1"]`
- **AND** version matches the REPL binary version

#### Scenario: Init with binary path
- **WHEN** user runs `synoema init myapp --ai claude --mcp-binary`
- **THEN** generated `.claude/settings.json` contains `"command": "~/.synoema/bin/synoema-mcp"`
- **AND** no npx dependency

## MODIFIED Requirements

### Requirement: Init MCP config generation
The `synoema init` command SHALL support `--mcp-binary` flag to generate configs pointing to local binary instead of npx.

#### Scenario: Default (npx) with version pin
- **WHEN** user runs `synoema init myapp --ai claude` (no --mcp-binary)
- **THEN** config uses `npx synoema-mcp@{version}`

#### Scenario: Binary mode
- **WHEN** user runs `synoema init myapp --ai claude --mcp-binary`
- **THEN** config uses absolute path to `~/.synoema/bin/synoema-mcp`
