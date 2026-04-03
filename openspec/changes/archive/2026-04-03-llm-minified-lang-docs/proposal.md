---
id: proposal
type: proposal
status: done
---

# Proposal: LLM Minified Language Reference

## Problem

LLMs have zero prior knowledge of Synoema. When asked to generate `.sno` code, they default
to Haskell/Python/OCaml syntax — producing invalid programs. A compact reference that fits
in a system prompt (~1500 tokens) would enable accurate first-pass code generation.

Key failure modes without a reference:
- Using `if/then/else` instead of `? -> :`
- Using `let/in` instead of indented bindings
- Using `def`/`fn` keywords instead of bare `name args = expr`
- Writing `[1, 2, 3]` instead of `[1 2 3]` (space-separated)
- Forgetting offside rule (indent = block)
- Not knowing ADT/typeclass syntax

## Goal

Produce `docs/llm/synoema.md` — a minified reference:
- ≤ 1800 tokens (cl100k_base) — fits in system prompt
- Covers every syntax construct with a one-line working example
- Highlights what's ABSENT (no `let`, no `if`, no `def`, no `,` in lists)
- Contrast table: "Haskell/Python → Synoema" for fast mental mapping
- Grouped for scanability, not depth

## Non-goals

- Formal type rules (already in language_reference.md)
- Tutorial narrative
- Duplication of ARCHITECTURE.md / RULES.md

## Output file

`docs/llm/synoema.md`
