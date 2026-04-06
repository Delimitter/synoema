# Synoema Compact Reference

BPE-aligned functional language. Files: `.sno`. Entry: `main = <expr>`.
Grammar: `lang/tools/constrained/synoema.gbnf`. Full stdlib: `docs/llm/stdlib.md`.

## Gotchas (READ FIRST)

| # | Rule |
|---|------|
| 1 | Cons pattern needs parens: `head (x:_) = x` not `head x:_ = x` |
| 2 | Lists space-separated: `[1 2 3]` never `[1, 2, 3]` |
| 3 | `:` is else-branch AND cons — protect cons in conditional: `? c -> (x:xs) : rest` |
| 4 | String concat: `++` not `+` — `"a" ++ "b"`, `+` is numbers only |
| 5 | No `return` — last expression is the result |
| 6 | `[f x]` = 2-element list — for single `[(f x)]` use parens |
| 7 | `show` converts to String: `show 42` = `"42"` |
| 8 | `/` on Int truncates: `10 / 3` = `3` |
| 9 | `index i xs` is 0-based, not `xs[i]` |
| 10 | Multiline: use `"\n"` in concat |
| 11 | `json_get` = single key, chain for nesting |
| 12 | `?` = if/else only, NOT pattern match — use multi-equation: `f (Just x) = ...` / `f None = ...` |
| 13 | `Maybe` NOT in prelude — define when needed. `Result` IS built-in |

## Operators (low → high)

`<-` bind | `|>` pipe | `||` or | `&&` and | `==` `!=` eq | `<` `>` `<=` `>=` cmp | `++` concat | `+` `-` add | `*` `/` `%` mul | `**` pow | `>>` compose | `.` field | `f x` apply (highest) | `\x -> e` lambda

## Core Rules

- `name args = body` — no def/fn keyword
- Pattern match via multiple equations: `f 0 = 1` / `f n = n * f (n-1)`
- Indentation = blocks (2-space), offside rule
- All bindings immutable, strict eval
- Types inferred (optional: `f : Int -> Int`)
- Define before use
- Conditional: `? cond -> then : else`
- Interpolation: `"x=${expr}"`

## Example 1: Recursion + Patterns

```sno
fac 0 = 1
fac n = n * fac (n - 1)

fizzbuzz n =
  ? n % 15 == 0 -> "FizzBuzz"
  : ? n % 3 == 0 -> "Fizz"
  : ? n % 5 == 0 -> "Buzz"
  : show n

main =
  result = map fizzbuzz [1..20]
  for_each print result
```

## Example 2: Lists + HOF + Records

```sno
dist {x, y} = x * x + y * y

move pt dx dy = {...pt, x = pt.x + dx, y = pt.y + dy}

main =
  points = [{x = 1, y = 2} {x = 3, y = 4} {x = 5, y = 0}]
  far = filter (\p -> dist p > 5) points
  dists = map (\p -> show (dist p)) far
  str_join ", " dists
```

## Stdlib

Lists: `map filter foldl length head tail reverse sum index take drop zip`
IO: `show print readline` | Strings: `str_len str_join str_trim`
Maps: `map_insert map_lookup map_get map_keys map_empty`
Result: `unwrap and_then is_ok map_ok` | JSON: `json_parse json_get`
