# Synoema: Strings & IO

BPE-aligned functional language. Files: `.sno`. Entry: `main = <expr>`.
Grammar: `lang/tools/constrained/synoema.gbnf`. Full stdlib: `docs/llm/stdlib.md`.

## Gotchas

| # | Rule |
|---|------|
| 4 | String concat: `++` not `+` — `"a" ++ "b"`, `+` is numbers only |
| 5 | No `return` — last expression is the result |
| 7 | `show` converts to String: `show 42` = `"42"` |
| 10 | Multiline: use `"\n"` in concat |

## Operators (low → high)

`<-` bind | `|>` pipe | `||` or | `&&` and | `==` `!=` eq | `++` concat | `+` `-` add | `*` `/` `%` mul | `.` field | `f x` apply (highest) | `\x -> e` lambda

## Core Rules

- String concat: `++` — `"hello" ++ " " ++ "world"`
- Interpolation: `"x = ${expr}"` (desugars to show + ++)
- IO bind: `line <- readline` | Sequence: `print "a" ; print "b"`
- `show` converts any value to String

## Example: String Processing

```sno
count_vowels s =
  chars = map (\i -> str_slice s i (i + 1)) [0..(str_len s - 1)]
  vowels = filter (\c -> c == "a" || c == "e" || c == "i" || c == "o" || c == "u") chars
  length vowels

main =
  s = "hello world"
  "vowels in '${s}': ${show (count_vowels s)}"
```

## Example: String Builder

```sno
greet name age =
  line1 = "Name: " ++ name
  line2 = "Age: " ++ show age
  line1 ++ "\n" ++ line2

main =
  msg = greet "Alice" 30
  print msg
```

## Stdlib

```
str_len         : String -> Int
str_slice       : String -> Int -> Int -> String  -- str from to
str_find        : String -> String -> Int -> Int  -- str sub start (-1=not found)
str_starts_with : String -> String -> Bool
str_trim        : String -> String
str_join        : String -> [String] -> String    -- separator list
show            : a -> String
print           : a -> ()
readline        : String
file_read       : String -> String                -- interpreter only
```
