# Tasks: MCP Auto-Deploy

## 1. MCP CLI flags (--version, --health)

- [x] 1.1 Add `--version` flag to `mcp/synoema-mcp/src/main.rs`: check `std::env::args()` before JSON-RPC loop, print `synoema-mcp {version}`, exit 0
- [x] 1.2 Add `--health` flag: print JSON `{"status":"ok","version":"...","protocol":"2024-11-05"}`, exit 0
- [x] 1.3 Add tests for both flags in `mcp/synoema-mcp/src/main.rs` (unit test: parse args → expected output)

## 2. Version pinning in init templates

- [x] 2.1 Change all MCP templates in `lang/templates/` to use `{{version}}` placeholder in args: `"synoema-mcp@{{version}}"` instead of `"synoema-mcp"`
- [x] 2.2 Update `init_project()` in `lang/crates/synoema-repl/src/main.rs`: pass version to template via new `apply_tmpl_versioned()` or extend `apply_tmpl()` to handle `{{version}}`
- [x] 2.3 Update `init_project_at()` in tests to match new template format
- [x] 2.4 Add test: init generates config with pinned version string

## 3. --mcp-binary flag in init

- [x] 3.1 Add `--mcp-binary` CLI flag parsing in main() alongside existing `--ai` flag
- [x] 3.2 In `init_project()`: if `--mcp-binary`, generate MCP configs with `"command": "{home}/.synoema/bin/synoema-mcp"` instead of npx
- [x] 3.3 Create binary-mode templates (or use conditional in existing templates): `mcp-*-binary.json.tmpl` or `{{mcp_command}}`/`{{mcp_args}}` placeholders
- [x] 3.4 Add test: init with --mcp-binary generates config with binary path

## 4. mcp-install command

- [x] 4.1 Add `mcp-install` subcommand parsing in main() (alongside run/jit/eval/init/etc.)
- [x] 4.2 Implement `mcp_install()` function: detect OS/arch → map to platform name (darwin-arm64, darwin-x64, linux-x64, win32-x64)
- [x] 4.3 Construct GitHub Release download URL: `https://github.com/Delimitter/synoema/releases/download/v{version}/synoema-mcp-{version}-{platform}{ext}`
- [x] 4.4 Implement download via `std::process::Command`: try curl → fallback wget → fallback PowerShell (Windows)
- [x] 4.5 Install binary to `~/.synoema/bin/synoema-mcp` (create dir, set executable permission on Unix)
- [x] 4.6 Support `--prefix <path>` flag for custom install location
- [x] 4.7 Support `--from-source` flag: run `cargo build --release` in mcp/ dir, copy binary
- [x] 4.8 Print success message with installed path and version
- [x] 4.9 Add test: platform detection logic (unit test with mock os/arch)

## 5. mcp-update command

- [x] 5.1 Add `mcp-update` subcommand parsing in main()
- [x] 5.2 Implement `mcp_update()`: find installed binary (check `~/.synoema/bin/`, then PATH)
- [x] 5.3 Get installed version: run `synoema-mcp --version` (depends on task 1.1)
- [x] 5.4 Query GitHub API: `https://api.github.com/repos/Delimitter/synoema/releases/latest` → parse `tag_name`
- [x] 5.5 Compare versions: if remote > installed → download and replace; else → "up to date"
- [x] 5.6 Add test: version comparison logic (unit test)

## 6. Cargo test clean

- [x] 6.1 Run `cargo test` in lang/ workspace — ensure 0 failures, 0 warnings
- [x] 6.2 Run `cargo test` in mcp/ workspace — ensure 0 failures, 0 warnings
