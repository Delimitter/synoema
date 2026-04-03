---
id: design
type: design
status: done
---

# Design: Synoema MCP Server

## Key decisions

### D1: Separate workspace at `mcp/`

`lang/` has a strict "no serde/tokio" rule. MCP needs serde_json for JSON-RPC.
Separate workspace avoids polluting the compiler's dependency tree.
`mcp/Cargo.toml` depends on `synoema-eval` and `synoema-types` via relative path.

Alternative: add feature-gated deps in lang/ — rejected, complicates Cargo.lock for compiler users.

### D2: Synchronous stdio, no tokio

MCP over stdio is request-response. One thread, read line → dispatch → write line.
No parallelism needed — the LLM sends one request at a time.
tokio would add ~500KB to binary and 10+ transitive deps for zero benefit.

### D3: `include_str!` for spec docs

Language reference and LLM ref are embedded at compile time.
Pros: single binary, always consistent.
Examples are read from disk at runtime (path relative to `SYNOEMA_ROOT` or exe).
Pros: examples evolve with the repo without recompiling the MCP server.

### D4: Tool errors as content, not JSON-RPC errors

MCP spec says: if the tool executed but produced an error (e.g. type error), return
`isError: true` in content. JSON-RPC error codes only for protocol-level failures.
This lets LLMs see the diagnostic in the tool result and self-correct.

### D5: `render_json` for diagnostics

`synoema_diagnostic::render_json` already exists and produces LLM-friendly structured
JSON. Use it directly. Don't invent a new error format.

### D6: `eval` wraps `eval_expr`, also shows type

`eval_expr` returns a Value. Separately call `typecheck` to get the type.
Combined output: `"{value} : {type}"` — same format as the REPL.
If typecheck fails (shouldn't if eval succeeded), show value only.

### D7: `SYNOEMA_ROOT` env var for path resolution

Examples and future resources need a path to the repo root.
Default: walk up from `std::env::current_exe()` looking for `lang/` directory.
Override: `SYNOEMA_ROOT=/path/to/synoema`.

### D8: Separate `prompts/codegen.md` file

The system prompt content lives in `mcp/prompts/codegen.md`, embedded via `include_str!`.
Keeping it separate from code makes it easy to edit without touching Rust.

## Module structure

```
main.rs         — arg parse (none), stdio loop, dispatch match
protocol.rs     — serde types: JsonRpcRequest, JsonRpcResponse, ToolDef, ResourceDef, etc.
tools.rs        — pub fn handle_tools_list(), handle_tools_call(name, args)
resources.rs    — pub fn handle_resources_list(), handle_resources_read(uri)
prompts.rs      — pub fn handle_prompts_list(), handle_prompts_get(name)
```

## Cargo.toml deps

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
synoema-eval  = { path = "../lang/crates/synoema-eval" }
synoema-types = { path = "../lang/crates/synoema-types" }
synoema-diagnostic = { path = "../lang/crates/synoema-diagnostic" }
```

No other deps. Binary size target: < 5MB.
