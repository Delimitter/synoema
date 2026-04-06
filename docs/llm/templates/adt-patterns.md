# Synoema: ADTs & Pattern Matching

BPE-aligned functional language. Files: `.sno`. Entry: `main = <expr>`.
Grammar: `lang/tools/constrained/synoema.gbnf`. Full stdlib: `docs/llm/stdlib.md`.

## Gotchas

| # | Rule |
|---|------|
| 1 | Cons pattern needs parens: `head (x:_) = x` not `head x:_ = x` |
| 5 | No `return` — last expression is the result |
| 12 | `?` = if/else only, NOT pattern match — use multi-equation functions |
| 13 | `Maybe` NOT in prelude — define when needed. `Result` IS built-in |

## Operators (low → high)

`|>` pipe | `||` or | `&&` and | `==` `!=` eq | `<` `>` `<=` `>=` cmp | `++` concat | `+` `-` add | `*` `/` `%` mul | `.` field | `f x` apply (highest) | `\x -> e` lambda

## Core Rules

- ADT: `TypeName = Con1 Type | Con2 Type Type | Con3`
- Pattern match via multiple equations (NOT `?`):
  `f (Con1 x) = ...` / `f (Con2 x y) = ...` / `f Con3 = ...`
- Constructor names start Uppercase, variables lowercase
- `derive (Show, Eq, Ord)` for auto instances

## Example: Shape ADT

```sno
Shape = Circle Int | Rect Int Int | Point

area (Circle r)  = r * r
area (Rect w h)  = w * h
area Point       = 0

describe s = "area=" ++ show (area s)

main =
  shapes = [(Circle 5) (Rect 4 6) Point]
  for_each (\s -> print (describe s)) shapes
```

## Example: Linked List

```sno
List a = Nil | Cons a (List a)

len Nil = 0
len (Cons _ xs) = 1 + len xs

toList [] = Nil
toList (x:xs) = Cons x (toList xs)

main = len (toList [1 2 3 4 5])
```

## Stdlib

`show : a -> String` | `print : a -> ()`
Result (prelude): `Ok a | Err e`
`unwrap : Result a e -> a` | `map_ok : (a->b) -> Result a e -> Result b e`
`and_then : (a -> Result b e) -> Result a e -> Result b e`
`is_ok : Result a e -> Bool` | `is_err : Result a e -> Bool`
