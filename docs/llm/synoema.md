# Synoema — LLM Quick Reference

**BPE-aligned functional language for LLM code generation.**
All 33 operators = exactly 1 cl100k_base token. 46% fewer tokens than Python.
Files: `.sno`. Entry point: `main = <expr>`.
Grammar: `lang/tools/constrained/synoema.gbnf` (use for constrained decoding).

---

## 1. Overrides

| Instead of | Synoema |
|------------|---------|
| `if c then x else y` | `? c -> x : y` |
| `let x = e in body` | indented `x = e` then `body` |
| `def f(x):` / `fn` / `fun` | `f x = body` |
| `return x` | last expression = value |
| `where` clause | local bindings indented before last expr |
| `case x of` | multi-equation `f Con = ...` |
| `data T = A \| B` | `T = A \| B` |
| `class` / `instance` | `trait` / `impl` |
| `import M` | `use M (f g)` or `use M (*)` |
| `[1, 2, 3]` | `[1 2 3]` — **no commas** |
| `x:xs` bare in pattern | `(x:xs)` — **parens required** |
| `s1 + s2` (strings) | `s1 ++ s2` — `+` = numbers only |
| `f"x={x}"` / `` `x=${x}` `` | `"x=${x}"` |
| `do { a; b }` | `a ; b` or `<-` bind |
| `// comment` | `-- comment` |
| `/// doc comment` | `--- doc comment` |

---

## 2. Axioms

- `name args = body` — no def keyword; patterns via multiple equations
- Indentation = blocks (2-space) — offside rule
- Expression-oriented — last expr = result
- All bindings immutable
- Strict eval, left-to-right
- Types inferred — annotations optional: `f : Int -> Int`

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

-- singleton/multi-element list pattern
only [x]    = x        -- matches exactly [x]
sum3 [a b c] = a+b+c   -- matches exactly 3 elements

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

-- punning: {x, y} ≡ {x = x, y = y}
point x y = {x, y}
mixed     = {x, y, sum = x + y}  -- mixed

-- record pattern (destructure in args)
dist {x, y} = x * x + y * y      -- pattern punning
dist2 {x = a, y = b} = a * a + b * b  -- explicit still works

-- record update: copy all fields, override listed
move_x pt dx = {...pt, x = pt.x + dx}    -- {...base, field = val}
```

---

## 7a. Maps (prelude)

```sno
-- sorted association list: Map k v = MkMap [Pair k v]
m = map_insert "b" 2 (map_insert "a" 1 map_empty)
map_lookup "a" m          -- Ok 1
map_get "x" 0 m           -- 0 (default)
map_keys m                -- ["a" "b"] (sorted)
from_pairs [(MkPair "x" 1) (MkPair "y" 2)]  -- build from list
```

---

## 7b. Type aliases

```sno
type Pos = {x : Int, y : Int}
type Transform = Int -> Int
type Pair a b = {fst : a, snd : b}

-- aliases are transparent (expanded at type check)
dist : Pos -> Int
dist p = p.x * p.x + p.y * p.y
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

-- derive: auto-generate typeclass instances
Color = Red | Green | Blue derive (Show, Eq, Ord)

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

## 10. Modules & Imports

```sno
mod Math
  square x = x * x
  pi = 3.14159

use Math (square pi)        -- selective import
use Math (*)                -- wildcard: import all exports

main = square 5             -- 25
```

Multi-file: `import "path.sno"` loads another file's modules/decls.

```sno
-- main.sno
import "math.sno"
use Math (square)
main = square 5
```

Circular imports → error. Diamond imports → loaded once.

---

## 11. String interpolation

```sno
msg = "Hello ${name}, you have ${count} items"
sum = "${a} + ${b} = ${a + b}"      -- exprs allowed
esc = "\$ is literal dollar"        -- \$ escapes
```

Desugars to `show` + `++`. No format specifiers.

---

## 12. IO & effects

```sno
main = print "hello"        -- print any value, returns ()

main = print "a" ; print "b"   -- sequence with ;

main =                      -- monadic bind
  line <- readline
  print line
```

---

## 13. Stdlib (summary — full list: `docs/llm/stdlib.md`)

| Function | Type | |
|----------|------|-|
| `show` | `a -> String` | any type to string |
| `print` | `a -> ()` | print + newline |
| `readline` | `String` | read line |
| `length` | `[a] -> Int` | list length |
| `map` | `(a->b) -> [a] -> [b]` | transform |
| `filter` | `(a->Bool) -> [a] -> [a]` | filter |
| `foldl` | `(b->a->b) -> b -> [a] -> b` | left fold |
| `concatMap` | `(a->[b]) -> [a] -> [b]` | map + flatten |
| `sum` | `[Int] -> Int` | sum list |
| `zip` | `[a] -> [b] -> [(a,b)]` | pair elements |
| `index` | `[a] -> Int -> a` | 0-based index |
| `take` `drop` | `Int -> [a] -> [a]` | first/skip n |
| `reverse` | `[a] -> [a]` | reverse list |
| `sqrt` `floor` `ceil` `abs` `round` | `Float -> Float` | math |
| `str_len` `str_slice` `str_find` `str_trim` | String ops | see stdlib.md |
| `map_insert` `map_lookup` `map_keys` | Map ops | sorted assoc list |
| `json_parse` `json_encode` `json_get` | JSON ops | Result-wrapped |
| `env` `env_or` | `String -> String` | env variables |
| `args` | `[String]` | CLI args after `--` |

---

## 14. Error handling (LLM feedback loop)

`--errors json` output includes `llm_hint`, `fixability`, `did_you_mean` fields.
Full reference: `docs/llm/error-feedback.md`

```bash
synoema --errors json run file.sno   # JSON with LLM hints
```

Feedback loop: `tools/llm/feedback_loop.py` — generate, check, retry with enriched errors.

---

## 15. Testing

```sno
--- example: fact 5 == 120          -- doctest (in doc comment)
test "base case" = fact 0 == 1      -- unit test (Bool expression)
test "inv" = prop xs -> reverse (reverse xs) == xs  -- property test
test "pos" = prop n -> fact n > 0 when n >= 0 && n <= 10  -- conditional
```

Keywords: `test` (declaration), `prop` (generator), `when` (conditional). Run: `synoema test file.sno`

---

## 16. Gotchas

1. **Cons pattern needs parens** — `head (x:_) = x` ✓  `head x:_ = x` ✗
2. **List: space-separated** — `[1 2 3]` never `[1, 2, 3]`
3. **`:` is both else-branch AND cons** — protect cons: `? c -> (x:xs) : rest`
4. **String concat `++`** — `+` is numeric only
5. **No `return`** — last expr is the result
