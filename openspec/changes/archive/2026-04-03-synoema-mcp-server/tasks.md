---
id: tasks
type: tasks
status: done
---

# Tasks: Synoema MCP Server

## MCP-1: Workspace + protocol skeleton

- [x] Create `mcp/Cargo.toml` (separate workspace, deps: serde+serde_json, path to synoema-eval/types/diagnostic)
- [x] Create `mcp/src/protocol.rs` — JsonRpcRequest, JsonRpcResponse, Tool, Resource, Prompt serde types
- [x] Create `mcp/src/main.rs` — stdin loop: read line → serde_json::from_str → dispatch → serde_json::to_string → println
- [x] Handle `initialize` → respond with capabilities (tools+resources+prompts)
- [x] Handle `notifications/initialized` → no-op (no response)
- [x] Handle unknown method → JSON-RPC error -32601

## MCP-2: Tools

- [x] Create `mcp/src/tools.rs`
- [x] `tools/list` → return 3 tools: eval, typecheck, run (with JSON schemas)
- [x] `tools/call` `eval` → `synoema_eval::eval_expr(code)` → format `"{val} : {type}"` or error
- [x] `tools/call` `typecheck` → `synoema_types::typecheck(code)` → return type of `main` or errors
- [x] `tools/call` `run` → `synoema_eval::eval_main(code)` → join stdout + value or error

## MCP-3: Resources

- [x] Create `mcp/src/resources.rs`
- [x] `resources/list` → return language_reference, llm_ref, examples URIs
- [x] `resources/read` `synoema://spec/language_reference` → `include_str!("../../docs/specs/language_reference.md")` (verify path exists)
- [x] `resources/read` `synoema://spec/llm_ref` → `include_str!("../../docs/llm/synoema.md")` (verify path exists)
- [x] `resources/read` `synoema://examples` → list `lang/examples/*.sno` filenames (runtime read via `SYNOEMA_ROOT`)
- [x] `resources/read` `synoema://examples/{name}` → read file content

## MCP-4: Prompts

- [x] Create `mcp/prompts/codegen.md` — system prompt: what Synoema is, 6 axioms, common errors, tool usage hint
- [x] Create `mcp/src/prompts.rs`
- [x] `prompts/list` → return `synoema_codegen` prompt
- [x] `prompts/get` `synoema_codegen` → return messages with embedded codegen.md content

## MCP-5: Verify build

- [x] `cargo build` in `mcp/` — 0 errors, 0 warnings
- [x] Manual smoke test: echo initialize request → correct response (`protocolVersion: 2024-11-05`)
- [x] Manual smoke test: eval tool → `42 : Int` for `6 * 7`
- [x] lang/ `cargo test` — 634 passed, 0 failed (fixed pre-existing Scope/Spawn stubs in types+eval+core)
