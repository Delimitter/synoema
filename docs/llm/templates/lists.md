# Synoema: Lists & Higher-Order Functions

BPE-aligned functional language. Files: `.sno`. Entry: `main = <expr>`.
Grammar: `lang/tools/constrained/synoema.gbnf`. Full stdlib: `docs/llm/stdlib.md`.

## Gotchas

| # | Rule |
|---|------|
| 1 | Cons pattern needs parens: `head (x:_) = x` not `head x:_ = x` |
| 2 | Lists space-separated: `[1 2 3]` never `[1, 2, 3]` |
| 3 | `:` is else-branch AND cons — protect cons: `? c -> (x:xs) : rest` |
| 5 | No `return` — last expression is the result |
| 6 | `[f x]` = 2-element list — for single `[(f x)]` use parens |

## Operators (low → high)

`|>` pipe | `||` or | `&&` and | `==` `!=` eq | `<` `>` `<=` `>=` cmp | `++` concat | `+` `-` add | `*` `/` `%` mul | `.` field | `f x` apply (highest) | `\x -> e` lambda

## Core Rules

- `name args = body` — no def/fn keyword
- Pattern match: `f [] = ...` / `f (x:xs) = ...` — PARENS on cons!
- Lists: `[1 2 3]` space-separated, `[1..10]` range, `x : xs` cons
- Comprehension: `[x*x | x <- xs, x > 3]`
- Pipe: `xs |> filter f |> map g`

## Example: Quicksort

```sno
qsort [] = []
qsort (x:xs) =
  lt = filter (\y -> y < x) xs
  ge = filter (\y -> y >= x) xs
  qsort lt ++ [x] ++ qsort ge

main = qsort [3 6 8 10 1 2 1]
```

## Example: Pipeline

```sno
main =
  nums = [1..20]
  evens = filter (\x -> x % 2 == 0) nums
  squared = map (\x -> x * x) evens
  foldl (\acc x -> acc + x) 0 squared
```

## Stdlib

```
map       : (a -> b) -> [a] -> [b]
filter    : (a -> Bool) -> [a] -> [a]
foldl     : (b -> a -> b) -> b -> [a] -> b
length    : [a] -> Int
head      : [a] -> a              -- error on []
tail      : [a] -> [a]
take      : Int -> [a] -> [a]
drop      : Int -> [a] -> [a]
reverse   : [a] -> [a]
zip       : [a] -> [b] -> [(a,b)]
index     : Int -> [a] -> a       -- 0-based
sum       : [Int] -> Int
concatMap : (a -> [b]) -> [a] -> [b]
```
