# Synoema: Arithmetic & Recursion

BPE-aligned functional language. Files: `.sno`. Entry: `main = <expr>`.
Grammar: `lang/tools/constrained/synoema.gbnf`. Full stdlib: `docs/llm/stdlib.md`.

## Gotchas

| # | Rule |
|---|------|
| 5 | No `return` — last expression is the result |
| 8 | `/` on Int truncates: `10 / 3` = `3` |
| 12 | `?` = if/else only, NOT pattern match — use multi-equation functions |

## Operators (low → high)

`|>` pipe | `||` or | `&&` and | `==` `!=` eq | `<` `>` `<=` `>=` cmp | `+` `-` add | `*` `/` `%` mul | `**` pow | `.` field | `f x` apply (highest) | `\x -> e` lambda

## Core Rules

- `name args = body` — no def/fn keyword
- Pattern match via multiple equations: `f 0 = 1` / `f n = n * f (n-1)`
- Indentation = blocks (2-space), offside rule
- All bindings immutable, strict eval
- Types inferred (optional: `f : Int -> Int`)
- Define before use
- Conditional: `? cond -> then : else`

## Example: Factorial + Power

```sno
fac 0 = 1
fac n = n * fac (n - 1)

pow _ 0 = 1
pow base exp = base * pow base (exp - 1)

main = show (fac 10) ++ ", " ++ show (pow 2 10)
```

## Example: FizzBuzz

```sno
fizzbuzz n =
  ? n % 15 == 0 -> "FizzBuzz"
  : ? n % 3 == 0 -> "Fizz"
  : ? n % 5 == 0 -> "Buzz"
  : show n

main = for_each (\n -> print (fizzbuzz n)) [1..30]
```

## Stdlib

`show : a -> String` | `print : a -> ()` | `abs : Int -> Int`
`even : Int -> Bool` | `odd : Int -> Bool` | `sqrt : Float -> Float`
`floor ceil round : Float -> Float` | `map filter foldl : list ops`
