---
id: proposal
type: proposal
status: done
---

# Proposal: Synoema MCP Server

## Problem

LLMs have no prior knowledge of Synoema. Before the language appears in training data, every
code generation session requires the LLM to infer syntax from scratch — producing invalid
programs (wrong operators, missing offside rules, Haskell/Python defaults).

Two failure modes:
1. **No reference** — LLM hallucinates syntax, can't self-correct without feedback
2. **No runtime** — LLM can't verify generated code, errors compound

## Goal

Create `mcp/` — a standalone MCP server (`synoema-mcp`) that exposes:

**Tools (active calls):**
- `eval` — evaluate a Synoema expression, returns value + inferred type + errors
- `typecheck` — typecheck a program, returns type or structured errors
- `run` — run a full `.sno` program, returns stdout + result

**Resources (passive context):**
- `synoema://spec/language_reference` — full language reference
- `synoema://spec/llm_ref` — minified LLM-optimized reference (≤1500 tokens)
- `synoema://examples` — list of available examples
- `synoema://examples/{name}` — specific example source

**Prompts:**
- `synoema_codegen` — system prompt for LLM code generation sessions

## Architecture constraint

The `lang/` workspace has a strict dependency rule: only Cranelift + pretty_assertions.
MCP requires `serde`/`serde_json` for JSON-RPC. Solution: `mcp/` is a **separate Cargo
workspace** at the repo root — not inside `lang/`. It depends on `synoema-eval` and
`synoema-types` via path.

## Non-goals

- No async runtime (tokio) — synchronous stdio loop is sufficient for MCP
- No HTTP transport — stdio only (standard MCP)
- No new language features — this is purely a tool wrapper

## Output

- `mcp/Cargo.toml` — separate workspace
- `mcp/src/main.rs` — JSON-RPC stdio loop
- `mcp/src/protocol.rs` — MCP types
- `mcp/src/tools.rs` — eval/typecheck/run implementations
- `mcp/src/resources.rs` — spec/examples serving
- `mcp/src/prompts.rs` — codegen system prompt
