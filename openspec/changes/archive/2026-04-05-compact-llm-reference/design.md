---
id: design
type: design
status: done
---

# Design: Compact LLM Reference

## Structure (gotcha-first ordering)

Research on small model attention patterns shows primacy bias — content at the start of context receives stronger attention. Therefore:

1. **Section 1: Critical Gotchas** (~200 tokens) — the 13 items that cause most LLM errors, in condensed table form
2. **Section 2: Operator Table** (~150 tokens) — all 33 operators, compact 3-column format
3. **Section 3: Core Axioms** (~100 tokens) — 8 rules (expression-oriented, immutable, define-before-use, etc.)
4. **Section 4: Example 1 — Recursive + Pattern Match** (~150 tokens) — factorial + fizzbuzz in one block
5. **Section 5: Example 2 — Lists + HOF + Records** (~150 tokens) — filter/map pipeline + record usage
6. **Section 6: Stdlib pointer** (~50 tokens) — "Full stdlib: docs/llm/stdlib.md" + top-10 most used functions

Total target: ~800 tokens (cl100k_base).

## Key Decisions

- **No ADT section** — examples show ADT usage inline, no separate section needed at this size
- **No modules/imports** — small models rarely need multi-file programs
- **No IO section** — `print` and `main =` shown in examples
- **No type classes** — advanced feature, not needed for 80% of tasks
- **No JSON section** — advanced, stdlib.md has full reference
- **Gotchas condensed to 1-line each** — "Cons: `(x:xs)` not `x:xs`" format
- **Examples chosen for max pattern coverage** — the 2 examples together cover: function def, pattern match, conditionals, lists, lambdas, pipes, records, stdlib (map, filter, foldl, show)

## Token Budget Allocation

| Section | Target tokens | Content |
|---------|--------------|---------|
| Header + meta | 30 | file ext, entry point, grammar path |
| Gotchas table | 200 | 13 items, 1-line each |
| Operators | 150 | 33 ops, compact table |
| Axioms | 100 | 8 core rules |
| Example 1 | 150 | recursive + pattern match |
| Example 2 | 150 | lists + HOF + records |
| Stdlib ref | 20 | pointer + top functions |
| **Total** | **~800** | |

## Verification

After creation, run: `python3 -c "import tiktoken; enc=tiktoken.get_encoding('cl100k_base'); print(len(enc.encode(open('docs/llm/synoema-compact.md').read())))"` to verify token count.
