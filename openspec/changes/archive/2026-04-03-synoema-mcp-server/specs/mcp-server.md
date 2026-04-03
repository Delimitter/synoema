---
id: mcp-server
type: delta-spec
status: done
capability: mcp-server
---

# Spec: Synoema MCP Server

## Protocol

MCP 2024-11-05 over stdio. Transport: newline-delimited JSON-RPC 2.0.

```
stdin  → {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{...}}
stdout ← {"jsonrpc":"2.0","id":1,"result":{...}}
```

Server is synchronous: reads one request, writes one response, loops.

## Initialization

`initialize` request → respond with:
```json
{
  "protocolVersion": "2024-11-05",
  "capabilities": {
    "tools": {},
    "resources": {},
    "prompts": {}
  },
  "serverInfo": { "name": "synoema", "version": "0.1.0" }
}
```

`notifications/initialized` → no response required.

## Tools

### `eval`
Evaluates a Synoema expression using the interpreter.

Input schema:
```json
{ "code": { "type": "string", "description": "Synoema expression or program fragment" } }
```

Output (success):
```json
{ "content": [{ "type": "text", "text": "42 : Int" }] }
```

Output (error):
```json
{ "content": [{ "type": "text", "text": "[E101] Type mismatch: ..." }], "isError": true }
```

Implementation: `synoema_eval::eval_expr(code)` → format as `{value} : {type}`.
If error: `synoema_diagnostic::render_json(diag)`.

### `typecheck`
Type-checks a full Synoema program.

Input schema:
```json
{ "code": { "type": "string", "description": "Full Synoema program source" } }
```

Output (success): `"main : Int"` (type of main binding)
Output (error): JSON diagnostic from `render_json`

Implementation: `synoema_types::typecheck(code)` → lookup `main` type.

### `run`
Runs a full Synoema program through the interpreter.

Input schema:
```json
{ "code": { "type": "string", "description": "Full Synoema program with main binding" } }
```

Output (success):
```json
{ "content": [{ "type": "text", "text": "Hello\n42" }] }
```
(stdout lines + final value, newline-joined)

Output (error): JSON diagnostic.

Implementation: `synoema_eval::eval_main(code)` → join output lines + value.

## Resources

### List: `resources/list`
Returns:
```json
[
  { "uri": "synoema://spec/language_reference", "name": "Language Reference", "mimeType": "text/markdown" },
  { "uri": "synoema://spec/llm_ref",            "name": "LLM Quick Reference",  "mimeType": "text/markdown" },
  { "uri": "synoema://examples",                "name": "Examples index",       "mimeType": "text/plain" }
]
```

### Read: `resources/read`

| URI | Source | Strategy |
|-----|--------|---------|
| `synoema://spec/language_reference` | `docs/specs/language_reference.md` | `include_str!` at compile time |
| `synoema://spec/llm_ref` | `docs/llm/synoema.md` | `include_str!` at compile time |
| `synoema://examples` | `lang/examples/*.sno` filenames | read dir at runtime |
| `synoema://examples/{name}` | `lang/examples/{name}.sno` | read file at runtime |

Path resolution: relative to the executable's location or `SYNOEMA_ROOT` env var.

## Prompts

### `synoema_codegen`
System prompt for LLM code generation sessions.

No arguments. Returns `messages: [{role: "user", content: "..."}]` with:
- What Synoema is (one sentence)
- 6 core syntax axioms
- Most common errors to avoid
- "Use the `eval` and `typecheck` tools to verify your code."

Content is `include_str!` from `mcp/prompts/codegen.md`.

## Error handling

All tool errors use `isError: true` in the content response, NOT JSON-RPC error codes.
This follows MCP convention: tool execution errors are content, not protocol errors.

JSON-RPC errors (code -32601 etc.) only for: unknown method, malformed request.

## File layout

```
mcp/
  Cargo.toml          — workspace: [synoema-mcp], deps: serde, serde_json
  prompts/
    codegen.md        — system prompt content
  src/
    main.rs           — stdio loop + dispatch
    protocol.rs       — Request/Response/Tool/Resource/Prompt types
    tools.rs          — eval/typecheck/run
    resources.rs      — spec/examples serving
    prompts.rs        — codegen prompt
```
