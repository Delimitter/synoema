---
id: proposal
type: proposal
status: done
---

# Compact LLM Reference

## Why

Current `docs/llm/synoema.md` is ~488 lines / ~1800 BPE tokens (cl100k_base). For small models (4B-14B) with 4K-8K context windows, 1800 tokens of reference leaves too little room for the actual task prompt + few-shot examples. Research shows smaller models benefit disproportionately from optimized context — they need every token to count.

Goal: create `docs/llm/synoema-compact.md` at ~800 tokens that retains maximum generation quality by prioritizing the information that prevents the most common errors.

## What Changes

- New file `docs/llm/synoema-compact.md` — ~800 token reference optimized for small models
- Gotcha-first ordering (most common errors at the top, where small models attend best)
- Operator table in compact form
- 2 minimal complete examples (covers pattern matching, lists, conditionals)
- Reference to full stdlib via `docs/llm/stdlib.md` path (not inlined)

## Capabilities

### New Capabilities

- `compact-llm-reference`: Condensed ~800-token reference document for small models (4B-32B), gotcha-first ordering, 2 complete examples

### Modified Capabilities

- None (original `synoema.md` stays unchanged)

## Scope

### In Scope

- Create `docs/llm/synoema-compact.md`
- Verify token count with tiktoken cl100k_base
- Ensure all 13 gotchas are preserved (condensed form)

### Out of Scope

- Modifying existing `docs/llm/synoema.md`
- Changes to compiler or language
- Benchmark testing (separate change)
