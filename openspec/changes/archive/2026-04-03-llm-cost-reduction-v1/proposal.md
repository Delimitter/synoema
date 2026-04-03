---
id: proposal
type: proposal
status: done
---

# Proposal: LLM Cost Reduction v1

## Problem

Synoema achieves 46% token savings vs Python but still has gaps that increase LLM usage cost:

1. **Weak LLMs hallucinate builtins** — no machine-readable stdlib catalog; small models invent non-existent functions
2. **Repeated types waste tokens** — no type aliases; record types duplicated across signatures
3. **Single-error-stop** — parser/type checker halt on first error; each fix = new LLM round = 2000+ tokens
4. **String building is verbose** — `"a" ++ show x ++ "b"` costs 3 extra tokens per concat vs interpolation
5. **Single-file ceiling** — no file imports; projects >200 LOC impossible; LLM must see entire codebase

## Goals

Implement 5 changes ordered by impact/cost ratio:

| # | Feature | Token impact | LLM cycle impact |
|---|---------|-------------|-----------------|
| 1 | Stdlib catalog (`docs/llm/stdlib.md`) | -30% errors for weak LLMs | Documentation only |
| 2 | Type aliases (`type Vec3 = {x: Int, y: Int, z: Int}`) | -20% on signatures | Parser + types + desugar |
| 3 | Error recovery (collect all errors in one pass) | — | -50% LLM fix cycles |
| 4 | String interpolation (`"x = ${show x}"`) | -15% on string code | Lexer + parser + desugar |
| 5 | Multi-file imports (`import "math.sno"`) | Enables >200 LOC projects | Parser + eval + codegen |

## Non-goals

- Memory model changes (Perceus RC) — separate change
- Stream fusion / lazy evaluation — separate change
- Array/Vector type — separate change
- LSP server — separate change

## Constraints

- Every new keyword/operator MUST be 1 BPE token (cl100k_base)
- `cargo test` clean (0 failures, 0 warnings) after each feature
- Interpreter-first, then JIT
- Update GBNF grammar for new syntax
- Update docs/llm/ for each new construct
