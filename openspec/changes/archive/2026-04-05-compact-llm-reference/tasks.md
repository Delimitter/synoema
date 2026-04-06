---
id: tasks
type: tasks
status: done
---

# Tasks: Compact LLM Reference

- [x] **T1: Create `docs/llm/synoema-compact.md`**
  - Write compact reference following design structure (gotcha-first)
  - Section 1: Gotchas table (13 items, 1-line each)
  - Section 2: Operators table (33 ops, compact)
  - Section 3: Core axioms (8 rules)
  - Section 4: Example 1 — recursive + pattern match (factorial, fizzbuzz)
  - Section 5: Example 2 — lists + HOF + records (filter/map pipeline)
  - Section 6: Stdlib pointer (top-10 + path to stdlib.md)

- [x] **T2: Verify token count**
  - 904 tokens (cl100k_base) — within target range
  - Trimmed operators section and redundant text to fit

- [x] **T3: Validate examples compile**
  - Example 1 (fac + fizzbuzz): parse ✓ typecheck ✓ run ✓ (correct output)
  - Example 2 (dist + records): parse ✓ typecheck ✓ run ✓ (outputs "25, 25")
