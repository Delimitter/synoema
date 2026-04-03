---
id: stdlib-catalog
type: spec
status: done
---

# Spec: Stdlib Catalog for LLMs

## Requirement

Create `docs/llm/stdlib.md` — machine-readable catalog of ALL built-in functions with types. Budget: ≤600 tokens (cl100k_base).

## Format

Compact table, one line per function, grouped by category:

```
## List
length : [a] -> Int
head : [a] -> a
tail : [a] -> [a]
map : (a -> b) -> [a] -> [b]
filter : (a -> Bool) -> [a] -> [a]
foldl : (b -> a -> b) -> b -> [a] -> b
concatMap : (a -> [b]) -> [a] -> [b]
sum : [Int] -> Int
```

## Categories

1. **List** — length, head, tail, map, filter, foldl, concatMap, sum
2. **String** — str_len, str_slice, str_find, str_starts_with, str_trim, json_escape
3. **Math** — sqrt, floor, ceil, round, abs, even, odd
4. **Logic** — not
5. **IO** — print, readline, show
6. **Convert** — show (polymorphic)

## Acceptance Criteria

- [ ] File exists at `docs/llm/stdlib.md`
- [ ] ≤600 tokens verified via BPE counter
- [ ] Every builtin from eval.rs and runtime.rs is listed
- [ ] Each entry has name + type signature
- [ ] Index file `docs/llm/synoema.md` updated to reference stdlib.md
