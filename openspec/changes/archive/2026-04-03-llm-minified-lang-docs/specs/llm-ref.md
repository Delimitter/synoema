---
id: llm-ref
type: delta-spec
status: done
capability: llm-minified-reference
---

# Spec: LLM Minified Reference Format

## Structure (ordered sections)

1. **Header** — one-line identity: what Synoema is
2. **Core syntax rules** — 5-6 bullet invariants (strict, immutable, offside, expression-oriented)
3. **Literals & operators** — compact table or one-liners
4. **Functions & pattern matching** — defining, calling, multi-equation
5. **Control flow** — conditional `? -> :`, local bindings
6. **Types & ADTs** — type definitions, constructors
7. **Type classes** — trait / impl
8. **Modules** — mod / use
9. **Lists & comprehensions** — `[a b c]`, `[x | x <- xs, guard]`, `[a..b]`
10. **Records** — `{f = v}`, `.field`, record patterns
11. **IO & effects** — `<-`, print, readline, `;` sequencing
12. **What's ABSENT** — no `if/let/def/fn/do/return/where`
13. **Contrast table** — Python/Haskell → Synoema for 10 common constructs

## Format rules

- Each construct: max 2 lines of code + 1 line comment (no prose paragraphs)
- Use `--` comments in code to explain inline
- Operators table: `op | meaning | assoc | prec` — condensed
- No formal notation (no `Γ ⊢`, no `∀`, no EBNF)
- Target: developer who knows Haskell or Python can read in 3 minutes

## Token budget

| Section | Target tokens |
|---------|--------------|
| Header + axioms | 80 |
| Operators table | 120 |
| Functions + patterns | 180 |
| Control + bindings | 80 |
| Types + ADTs | 120 |
| Typeclasses | 80 |
| Modules | 60 |
| Lists + records | 100 |
| IO | 60 |
| Absent + contrast | 150 |
| **Total** | **~1030** |

Target: comfortably under 1500 cl100k tokens.
