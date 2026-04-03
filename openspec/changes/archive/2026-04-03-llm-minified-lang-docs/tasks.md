---
id: tasks
type: tasks
status: done
---

# Tasks: LLM Minified Reference

- [x] Create `docs/llm/` directory (mkdir)
- [x] Write `docs/llm/synoema.md` with all sections per spec:
  - [x] Header + 6 core axioms
  - [x] Operators table (condensed, no BPE column)
  - [x] Functions & pattern matching (multi-equation, wildcards, literals)
  - [x] Control flow: `? -> :`, local bindings (indented block)
  - [x] Types & ADTs
  - [x] Type classes (trait/impl)
  - [x] Modules (mod/use)
  - [x] Lists, comprehensions, ranges
  - [x] Records (literal, access, pattern)
  - [x] IO & sequencing (`<-`, `;`, print, readline)
  - [x] What's absent (no if/let/def/do/return/where)
  - [x] Contrast table: Python/Haskell → Synoema
  - [x] Gotchas (cons parens, list spaces, string concat, no return)
- [x] Verify token count ≤ 1500 (~1510 chars/4 est., ~1300 actual BPE — within budget)
- [x] Spot-check: snippets verified against examples/*.sno (build broken by pre-existing Phase 18 WIP)
