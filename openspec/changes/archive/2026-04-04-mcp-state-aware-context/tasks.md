# Tasks: MCP State-Aware Context

## Implementation

- [x] 1. Create `mcp/synoema-mcp/src/state.rs` — AppState enum, StateTracker struct, transition logic, baseline_context(), thread_local global
- [x] 2. Integrate StateTracker into `main.rs` — call `state::on_tool_result()` after each tool call in `handle_tools_call`
- [x] 3. Add `get_context` and `get_state` tools to `tools.rs` — definitions in `list()`, dispatch in `call()`
- [x] 4. Add tests in `state.rs` — transition table coverage, baseline content assertions
- [x] 5. Update `docs/mcp.md` — document new tools `get_context` and `get_state`
- [x] 6. Run `cargo test -p synoema-mcp` and `cargo build -p synoema-mcp` — 0 failures, 0 warnings
