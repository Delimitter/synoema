---
id: design
type: design
status: done
---

# Design: Task-Specific RAG Templates

## Template Structure

Each template follows the same layout:

```
[compact header: file ext, entry point, grammar]  (~30 tok)
[category-specific gotchas]                        (~80-120 tok)
[operator table — SAME across all templates]       (~150 tok)
[core axioms]                                      (~80 tok)
[1-2 category-specific examples]                   (~200-250 tok)
[relevant stdlib signatures]                       (~100-150 tok)
```

Target: ~650-750 tokens per template (leaves more room for task prompt than 800-tok compact ref).

## 5 Categories

| Category | File | Key Features | Gotchas Included |
|----------|------|-------------|------------------|
| **arithmetic** | `arithmetic.md` | recursion, pattern match on numbers, conditionals | #5 no return, #8 int division |
| **lists** | `lists.md` | cons, comprehensions, map/filter/foldl, ranges | #1 cons parens, #2 space-sep, #3 colon ambiguity, #6 [f x] trap |
| **adt-patterns** | `adt-patterns.md` | ADT definition, constructor patterns, multi-equation | #12 ? is not pattern match, #13 Maybe not in prelude |
| **records-maps** | `records-maps.md` | records, punning, update, Map stdlib | #11 json_get flat key |
| **string-io** | `string-io.md` | string ops, interpolation, IO, print/readline | #4 ++ not +, #7 show, #10 multiline |

## Gotcha Injection Map

```json
{
  "lists": [1, 2, 3, 6],
  "strings": [4, 7, 10],
  "pattern_match": [1, 12],
  "records": [11],
  "adt": [12, 13],
  "io": [5, 7],
  "numbers": [5, 8],
  "json": [11, 13]
}
```

Each gotcha ID maps to the numbered gotcha from synoema.md section 16. The map is stored as `docs/llm/templates/gotcha-map.json`.

## Example Selection Per Category

| Category | Example 1 | Example 2 |
|----------|-----------|-----------|
| arithmetic | factorial (recursion + pattern) | fizzbuzz (nested conditional) |
| lists | quicksort (cons, filter, concat) | filter-map pipeline (HOF + range) |
| adt-patterns | Shape ADT + area function | linked list type + len/append |
| records-maps | point record + move/dist | word frequency counter (Map ops) |
| string-io | string processing pipeline | interactive IO (readline + print) |

## Stdlib Signatures Per Category

Each template includes only the stdlib functions relevant to its category:

- **arithmetic**: show, print, abs, even, odd, sqrt
- **lists**: map, filter, foldl, length, head, tail, take, drop, reverse, zip, index, concatMap, sum
- **adt-patterns**: show, print (construction/destruction shown in examples)
- **records-maps**: map_insert, map_lookup, map_get, map_keys, map_values, has_key, map_empty, from_pairs
- **string-io**: str_len, str_slice, str_find, str_trim, str_join, show, print, readline, file_read
