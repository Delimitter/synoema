# Synoema: Records & Maps

BPE-aligned functional language. Files: `.sno`. Entry: `main = <expr>`.
Grammar: `lang/tools/constrained/synoema.gbnf`. Full stdlib: `docs/llm/stdlib.md`.

## Gotchas

| # | Rule |
|---|------|
| 5 | No `return` — last expression is the result |
| 11 | `json_get` = single flat key, chain for nesting |

## Operators (low → high)

`|>` pipe | `||` or | `&&` and | `==` `!=` eq | `<` `>` `<=` `>=` cmp | `++` concat | `+` `-` add | `*` `/` `%` mul | `.` field | `f x` apply (highest) | `\x -> e` lambda

## Core Rules

- Record literal: `{x = 3, y = 4}` | Punning: `{x, y}` = `{x = x, y = y}`
- Field access: `pt.x` | Update: `{...pt, x = pt.x + 1}`
- Pattern: `dist {x, y} = x*x + y*y`
- Map = sorted assoc list (prelude)

## Example: Records

```sno
dist {x, y} = x * x + y * y

move pt dx dy = {...pt, x = pt.x + dx, y = pt.y + dy}

main =
  p = {x = 3, y = 4}
  p2 = move p 1 2
  show (dist p) ++ ", " ++ show (dist p2)
```

## Example: Word Frequency

```sno
count_word word counts =
  n = map_get word 0 counts
  map_insert word (n + 1) counts

main =
  words = ["the" "cat" "sat" "on" "the" "mat" "the"]
  counts = foldl (\acc w -> count_word w acc) map_empty words
  keys = map_keys counts
  for_each (\k -> print (k ++ ": " ++ show (map_get k 0 counts))) keys
```

## Stdlib

```
map_empty     : Map k v
map_insert    : k -> v -> Map k v -> Map k v
map_lookup    : k -> Map k v -> Result v String
map_get       : k -> v -> Map k v -> v          -- with default
has_key       : k -> Map k v -> Bool
map_delete    : k -> Map k v -> Map k v
map_keys      : Map k v -> [k]                  -- sorted
map_values    : Map k v -> [v]
from_pairs    : [Pair k v] -> Map k v
```
