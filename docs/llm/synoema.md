# Synoema — LLM Quick Reference

**BPE-aligned functional language for LLM code generation.**
All 33 operators = exactly 1 cl100k_base token. 46% fewer tokens than Python.
Files: `.sno`. Entry point: `main = <expr>`.
Grammar: `lang/tools/constrained/synoema.gbnf` (use for constrained decoding).

---

## 1. Mental model: what NOT to write

Override your Haskell/Python priors first:

| Instead of (Haskell/Python) | Write in Synoema | Why |
|------------------------------|------------------|-----|
| `if c then x else y` | `? c -> x : y` | 3 tokens, no keywords |
| `let x = e in body` | indented block: `x = e` then `body` | offside rule |
| `def f(x):` / `fn x ->` / `fun` | `f x = body` | no def keyword |
| `return x` | last expression = value | expression-oriented |
| `where` clause | local bindings indented before last expr | — |
| `case x of` | multi-equation `f Con = ...` | — |
| `data T = A \| B` | `T = A \| B` | no `data` |
| `class` / `instance` | `trait` / `impl` | — |
| `import M` | `use M (f g)` | selective, explicit |
| `[1, 2, 3]` | `[1 2 3]` | **no commas** |
| `x:xs` bare in pattern | `(x:xs)` | **parens required** |
| `s1 + s2` (strings) | `s1 ++ s2` | `+` = numbers only |
| `do { a; b }` | `a ; b` or `<-` bind | — |
| `// comment` | `-- comment` | — |

---

## 2. Core axioms

- **No def keyword** — `name args = body`; patterns via multiple equations
- **Offside rule** — indentation defines blocks (2-space), like Python/Haskell
- **Expression-oriented** — every construct returns a value; last expr = result
- **Immutable** — all bindings are immutable
- **Strict** — eager evaluation, left-to-right; no lazy thunks
- **Types inferred** — annotations optional; `f : Int -> Int` is documentation

---

## 3. Operators (precedence low → high)

| Op | Meaning | Prec | Assoc |
|----|---------|------|-------|
| `<-` | bind (IO) | 1 | right |
| `\|>` | pipe: `x \|> f` = `f x` | 2 | left |
| `\|\|` | logical or | 3 | left |
| `&&` | logical and | 4 | left |
| `==` `!=` | equality | 5 | none |
| `<` `>` `<=` `>=` | comparison | 6 | none |
| `++` | string/list concat | 7 | right |
| `+` `-` | add/sub | 8 | left |
| `*` `/` `%` | mul/div/mod | 9 | left |
| `**` | power | 9 | right |
| `>>` | compose: `f >> g` = `\x -> g (f x)` | 10 | right |
| `-` (prefix) | negate | 11 | — |
| `.` | field access | 12 | left |
| juxtaposition | apply `f x` (highest) | 13 | left |

---

## 4. Functions & pattern matching

```sno
-- definition
double x = x * 2

-- multi-equation: top-to-bottom, first match wins
fac 0 = 1
fac n = n * fac (n - 1)

-- wildcard, literal patterns
describe 0 = "zero"
describe _ = "other"

-- cons pattern (always parenthesise)
head (x:_)  = x
tail (_:xs) = xs
isEmpty []  = true
isEmpty _   = false

-- lambda
square = \x -> x * x

-- type annotation (optional)
add : Int -> Int -> Int
add x y = x + y
```

---

## 5. Control flow & local bindings

```sno
-- conditional: ? cond -> then : else
abs x = ? x < 0 -> -x : x

-- nested conditional (multiline ok)
fizzbuzz n =
  ? n % 15 == 0 -> "FizzBuzz"
  : ? n % 3 == 0 -> "Fizz"
  : ? n % 5 == 0 -> "Buzz"
  : show n

-- local bindings: indented block, last line = result
hyp a b =
  a2 = a * a
  b2 = b * b
  a2 + b2

-- pipe chain (idiomatic for data transforms)
result = [1..10] |> filter (\x -> x % 2 == 0) |> map (\x -> x * x)
```

---

## 6. Lists

```sno
xs    = [1 2 3 4 5]   -- space-separated, no commas
empty = []
r     = [1..10]       -- range (inclusive)
both  = [1 2] ++ [3 4]

-- cons
xs = 1 : [2 3]        -- builds [1 2 3]

-- comprehension: [expr | var <- source, guard]
evens = [x | x <- [1..20], x % 2 == 0]

-- stdlib
doubled = map    (\x -> x * 2)           [1 2 3]
odds    = filter (\x -> x % 2 != 0)     [1 2 3 4 5]
total   = foldl  (\acc x -> acc + x) 0  [1 2 3]
```

---

## 7. Records

```sno
pt     = {x = 3, y = 4}          -- literal
px     = pt.x                    -- field access
circle = {center = {x=0, y=0}, radius = 5}  -- nested

-- record pattern (destructure in args)
dist {x = a, y = b} = a * a + b * b

-- "update": create new record
move_x pt dx = {x = pt.x + dx, y = pt.y}
```

---

## 8. ADTs & pattern matching

```sno
-- type definition
Maybe a = Just a | None
Shape   = Circle Float | Rect Float Float | Point

-- construction
x = Just 42
s = Circle 3.0

-- pattern matching on constructors
fromMaybe def None     = def
fromMaybe _   (Just x) = x

area (Circle r)  = r * r
area (Rect w h)  = w * h
area Point       = 0
```

---

## 9. Type classes

```sno
trait Show a
  show : a -> String

-- implement
Color = Red | Green | Blue

impl Show Color
  show Red   = "red"
  show Green = "green"
  show Blue  = "blue"

-- constrained impl
impl Show (Maybe a) ? Show a
  show None     = "None"
  show (Just x) = "Just " ++ show x

main = show (Just 42)   -- "Just 42"
```

---

## 10. Modules

```sno
mod Math
  square x = x * x
  pi = 3.14159

use Math (square pi)        -- selective import, must list names

main = square 5             -- 25
```

---

## 11. IO & effects

```sno
main = print "hello"        -- print any value, returns ()

main = print "a" ; print "b"   -- sequence with ;

main =                      -- monadic bind
  line <- readline
  print line
```

---

## 12. Stdlib

| Function | Type | |
|----------|------|-|
| `show` | `a -> String` | any type to string |
| `print` | `a -> ()` | print + newline |
| `readline` | `String` | read line |
| `length` | `[a] -> Int` | list length |
| `map` | `(a->b) -> [a] -> [b]` | transform |
| `filter` | `(a->Bool) -> [a] -> [a]` | filter |
| `foldl` | `(b->a->b) -> b -> [a] -> b` | left fold |
| `sqrt` `floor` `ceil` `abs` `round` | `Float -> Float` | math |

---

## 13. Gotchas

1. **Cons pattern needs parens** — `head (x:_) = x` ✓  `head x:_ = x` ✗
2. **List: space-separated** — `[1 2 3]` never `[1, 2, 3]`
3. **`:` is both else-branch AND cons** — protect cons: `? c -> (x:xs) : rest`
4. **String concat `++`** — `+` is numeric only
5. **No `return`** — last expr is the result
