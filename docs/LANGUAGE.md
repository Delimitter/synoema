# Synoema Language Guide

A practical reference for writing Synoema programs. For the formal specification, see [language_reference.md](specs/language_reference.md). For the LLM-optimized quick reference, see [llm/synoema.md](llm/synoema.md).

---

## Mental Model

If you know Python or Haskell, override these habits first:

| Instead of | Write in Synoema | Why |
|------------|------------------|-----|
| `def f(x):` | `f x = body` | No `def` keyword |
| `if c then x else y` | `? c -> x : y` | 3 tokens instead of keywords |
| `return x` | *(just the expression)* | Last expression = result |
| `[1, 2, 3]` | `[1 2 3]` | Space-separated, no commas |
| `let x = e in body` | indent `x = e` then `body` | Offside rule (indentation) |
| `data T = A \| B` | `T = A \| B` | No `data` keyword |
| `class` / `instance` | `trait` / `impl` | |
| `import M` | `use M (f g)` | Selective imports |
| `s1 + s2` (strings) | `s1 ++ s2` | `+` is for numbers only |
| `f"x={x}"` | `"x=${x}"` | `${}` interpolation |

**Core axioms:**
- **No `def`, no `return`** — `name args = body`; last expression is the result
- **Offside rule** — indentation defines blocks (like Python)
- **Immutable** — all bindings are immutable
- **Strict** — eager evaluation, left-to-right (not lazy like Haskell)
- **Types inferred** — annotations are optional documentation: `f : Int -> Int`

---

## Functions & Pattern Matching

Functions are defined with `name args = body`. Multiple equations define pattern matching (first match wins, top to bottom):

```sno
-- Simple function
double x = x * 2

-- Pattern matching: multiple equations
fac 0 = 1
fac n = n * fac (n - 1)

-- Wildcard pattern
describe 0 = "zero"
describe _ = "other"

-- Cons pattern (parentheses required!)
head (x:_)  = x
tail (_:xs) = xs
isEmpty []  = true
isEmpty _   = false

-- Lambda
square = \x -> x * x

-- Type annotation (optional)
add : Int -> Int -> Int
add x y = x + y
```

> **Gotcha:** cons patterns MUST be parenthesized — `head (x:_) = x` works, `head x:_ = x` does not.

---

## Types

Synoema has Hindley-Milner type inference. You rarely need to write types, but you can annotate for documentation.

### Primitive types

| Type | Examples | Notes |
|------|----------|-------|
| `Int` | `42`, `-7`, `0` | 63-bit signed integer |
| `Float` | `3.14`, `2.0` | 64-bit IEEE 754 |
| `Bool` | `true`, `false` | |
| `String` | `"hello"`, `"x=${x}"` | With `${}` interpolation |
| `()` | `()` | Unit type |

### Composite types

| Type | Example | Notes |
|------|---------|-------|
| `List a` | `[1 2 3]`, `[]` | Homogeneous, space-separated |
| Records | `{x = 3, y = 4}` | Structural (row polymorphism) |
| ADT | `Maybe a = Just a \| None` | Algebraic data types |
| `Result a e` | `Ok 42`, `Err "fail"` | From prelude |

### Type aliases

```sno
type Pos = {x : Int, y : Int}
type Transform = Int -> Int
type Pair a b = {fst : a, snd : b}
```

Aliases are transparent — expanded at type check time.

---

## Operators

Precedence from lowest to highest:

| Op | Meaning | Assoc |
|----|---------|-------|
| `<-` | IO bind | right |
| `\|>` | pipe: `x \|> f` = `f x` | left |
| `\|\|` | logical or | left |
| `&&` | logical and | left |
| `==` `!=` | equality | none |
| `<` `>` `<=` `>=` | comparison | none |
| `++` | string/list concat | right |
| `+` `-` | add/sub | left |
| `*` `/` `%` | mul/div/mod | left |
| `**` | power | right |
| `>>` | compose: `f >> g` | right |
| `-` (prefix) | negate | — |
| `.` | field access | left |
| juxtaposition | function application `f x` | left |

Every operator is exactly 1 BPE token (cl100k_base).

---

## Control Flow

### Conditional: `? -> :`

The ternary operator replaces `if/else`:

```sno
abs x = ? x < 0 -> -x : x
```

Chain for multi-branch:

```sno
fizzbuzz n =
  ? n % 15 == 0 -> "FizzBuzz"
  : ? n % 3 == 0 -> "Fizz"
  : ? n % 5 == 0 -> "Buzz"
  : show n
```

Where-bindings work in both then and else branches of ternary:

```sno
classify x =
  ? x > 0 -> label
    label = "positive"
  : label
    label = "non-positive"
```

### Local bindings

Indented lines before the final expression create local bindings:

```sno
hyp a b =
  a2 = a * a
  b2 = b * b
  a2 + b2
```

### Pipes

`|>` passes the left side as the last argument to the right side:

```sno
result = [1..10]
  |> filter (\x -> x % 2 == 0)
  |> map (\x -> x * x)
  |> sum
-- result = 220
```

---

## Lists

```sno
xs    = [1 2 3 4 5]       -- space-separated, NO commas
empty = []
r     = [1..10]            -- range (inclusive): [1 2 3 ... 10]
both  = [1 2] ++ [3 4]    -- concatenation: [1 2 3 4]

-- Singleton and multi-element list patterns
only [x]     = x           -- matches exactly one element
sum3 [a b c] = a + b + c   -- matches exactly three elements

-- Cons (prepend)
ys = 1 : [2 3]             -- [1 2 3]

-- List comprehension
evens = [x | x <- [1..20] , x % 2 == 0]

-- Stdlib operations
doubled = map (\x -> x * 2) [1 2 3]           -- [2 4 6]
odds    = filter (\x -> x % 2 != 0) [1..5]    -- [1 3 5]
total   = foldl (\acc x -> acc + x) 0 [1..5]  -- 15
```

> **Gotcha:** `:` is both the cons operator and the else-branch in `?->:`. In a then-branch, parenthesize cons: `? cond -> (x:xs) : rest`

---

## Records

```sno
-- Create
pt = {x = 3, y = 4}

-- Field access
px = pt.x                          -- 3

-- Nested records
circle = {center = {x = 0, y = 0}, radius = 5}
cx = circle.center.x               -- 0

-- Punning: {x, y} = {x = x, y = y}
point x y = {x, y}

-- Pattern matching (destructure in function args)
dist {x, y} = x * x + y * y

-- Record update (spread syntax)
move_x pt dx = {...pt, x = pt.x + dx}
```

---

## Algebraic Data Types (ADTs)

```sno
-- Define a sum type
Maybe a = Just a | None
Shape = Circle Float | Rect Float Float | Point

-- Construct values
x = Just 42
s = Circle 3.0

-- Pattern match via multiple equations
fromMaybe def None     = def
fromMaybe _   (Just x) = x

area (Circle r)  = r * r
area (Rect w h)  = w * h
area Point       = 0
```

### Derive

Auto-generate typeclass instances:

```sno
Color = Red | Green | Blue derive (Show, Eq, Ord)

main = show Red              -- "Red"
```

---

## Type Classes

```sno
-- Define a trait
trait Show a
  show : a -> String

-- Implement for a type
Color = Red | Green | Blue

impl Show Color
  show Red   = "red"
  show Green = "green"
  show Blue  = "blue"

-- Constrained implementation
impl Show (Maybe a) ? Show a
  show None     = "None"
  show (Just x) = "Just " ++ show x

main = show (Just 42)   -- "Just 42"
```

---

## Modules

### Single-file modules

```sno
mod Math
  square x = x * x
  pi = 3.14159

use Math (square pi)        -- selective import
use Math (*)                -- wildcard: import all

main = square 5             -- 25
```

### Multi-file imports

```sno
-- main.sno
import "math.sno"
use Math (square)
main = square 5
```

```sno
-- math.sno
mod Math
  square x = x * x
```

Circular imports produce an error. Diamond imports load each file once.

---

## String Interpolation

```sno
name = "world"
msg = "Hello ${name}"              -- "Hello world"
sum = "${a} + ${b} = ${a + b}"     -- expressions allowed
esc = "\$ is literal dollar"       -- escape with \$
```

Desugars to `show` + `++`. No format specifiers.

---

## IO & Effects

```sno
-- Print to stdout
main = print "hello"

-- Sequence with ;
main = print "a" ; print "b"

-- Read from stdin (monadic bind with <-)
main =
  line <- readline
  print ("You said: " ++ line)
```

### File I/O (interpreter only)

```sno
main =
  fd = fd_open "data.txt"
  line = fd_readline fd
  fd_close fd
  print line
```

---

## Standard Library

### List operations

| Function | Type | Description |
|----------|------|-------------|
| `length` | `[a] -> Int` | List length |
| `head` | `[a] -> a` | First element (error on `[]`) |
| `tail` | `[a] -> [a]` | All but first (error on `[]`) |
| `sum` | `[Int] -> Int` | Sum of elements |
| `map` | `(a -> b) -> [a] -> [b]` | Transform each element |
| `filter` | `(a -> Bool) -> [a] -> [a]` | Keep matching elements |
| `foldl` | `(b -> a -> b) -> b -> [a] -> b` | Left fold |
| `concatMap` | `(a -> [b]) -> [a] -> [b]` | Map and flatten |
| `zip` | `[a] -> [b] -> [(a, b)]` | Pair elements (stops at shorter) |
| `index` | `Int -> [a] -> a` | 0-based index (error on out-of-bounds) |
| `take` | `Int -> [a] -> [a]` | First n elements |
| `drop` | `Int -> [a] -> [a]` | Skip first n elements |
| `reverse` | `[a] -> [a]` | Reverse list |

### String operations

| Function | Type | Description |
|----------|------|-------------|
| `str_len` | `String -> Int` | String length |
| `str_slice` | `String -> Int -> Int -> String` | Substring (from, to) |
| `str_find` | `String -> String -> Int -> Int` | Find substring (-1 = not found) |
| `str_starts_with` | `String -> String -> Bool` | Prefix check |
| `str_trim` | `String -> String` | Trim whitespace |
| `json_escape` | `String -> String` | Escape for JSON |

### Math

| Function | Type | Description |
|----------|------|-------------|
| `sqrt` | `Float -> Float` | Square root |
| `floor` | `Float -> Float` | Round down |
| `ceil` | `Float -> Float` | Round up |
| `round` | `Float -> Float` | Round to nearest |
| `abs` | `Int -> Int` | Absolute value |
| `even` / `odd` | `Int -> Bool` | Parity check |

### IO

| Function | Type | Description |
|----------|------|-------------|
| `print` | `a -> ()` | Print with newline |
| `show` | `a -> String` | Convert any type to string |
| `readline` | `String` | Read line from stdin |
| `not` | `Bool -> Bool` | Logical negation |

### Result type (from prelude)

```sno
Result a e = Ok a | Err e
```

| Function | Type | Description |
|----------|------|-------------|
| `map_ok` | `(a -> b) -> Result a e -> Result b e` | Transform Ok value |
| `map_err` | `(e -> f) -> Result a e -> Result a f` | Transform Err value |
| `unwrap` | `Result a e -> a` | Extract Ok (error on Err) |
| `unwrap_or` | `a -> Result a e -> a` | Extract Ok with default |
| `is_ok` | `Result a e -> Bool` | Check if Ok |
| `is_err` | `Result a e -> Bool` | Check if Err |
| `and_then` | `(a -> Result b e) -> Result a e -> Result b e` | Chain operations |
| `error` | `String -> a` | Runtime panic |

### File / Network (interpreter only)

| Function | Type | Description |
|----------|------|-------------|
| `file_read` | `String -> String` | Read entire file |
| `fd_open` | `String -> Fd` | Open for reading |
| `fd_open_write` | `String -> Fd` | Open for writing |
| `fd_readline` | `Fd -> String` | Read one line |
| `fd_write` | `Fd -> String -> ()` | Write string |
| `fd_close` | `Fd -> ()` | Close handle |
| `tcp_listen` | `Int -> Fd` | Listen on port |
| `tcp_accept` | `Fd -> Fd` | Accept connection |
| `fd_popen` | `String -> Fd` | Run command, get stdout |

### Concurrency (interpreter only)

| Function | Type | Description |
|----------|------|-------------|
| `chan` | `Chan a` | Create typed channel |
| `send` | `Chan a -> a -> ()` | Send value |
| `recv` | `Chan a -> a` | Receive value (blocking) |

---

## Testing

Synoema has built-in test support. Run tests with `synoema test file.sno` or `synoema test directory/`.

### Doctests

In doc comments (`---`), write expected results:

```sno
--- Compute factorial.
--- example: fact 5 == 120
fact 0 = 1
fact n = n * fact (n - 1)
```

### Unit tests

```sno
test "base case" = fact 0 == 1
test "fact 10"   = fact 10 == 3628800
```

### Property tests

```sno
test "reverse involution" = prop xs -> reverse (reverse xs) == xs
test "positive factorial" = prop n -> fact n >= 1 when n >= 0 && n <= 10
```

Keywords: `test` (declare test), `prop` (random input), `when` (precondition).

---

## Error Handling

Use the `Result` type from the prelude for recoverable errors:

```sno
safe_div x 0 = Err "division by zero"
safe_div x y = Ok (x / y)

main =
  r = safe_div 10 3
  result = map_ok (\x -> x * 2) r
  show result                        -- "Ok(6)"
```

Chain with `and_then`:

```sno
parse_and_double input =
  parse input |> and_then (\n -> safe_div n 2)
```

For unrecoverable errors, use `error`:

```sno
head []    = error "empty list"
head (x:_) = x
```

### Structured error output

For LLM toolchains, use `--errors json` for machine-readable diagnostics:

```bash
synoema --errors json run file.sno
```

Output includes `llm_hint`, `fixability`, and `did_you_mean` fields. See [llm/error-feedback.md](llm/error-feedback.md).

---

## LLM Integration

### MCP Server

Connect Synoema to Claude Desktop, Cursor, or Zed:

```bash
npx synoema-mcp   # no install required
```

Add to your MCP config:
```json
{
  "mcpServers": {
    "synoema": { "command": "npx", "args": ["synoema-mcp"] }
  }
}
```

Full MCP documentation: [mcp.md](mcp.md)

### Constrained Decoding

Synoema includes a GBNF grammar for guaranteed syntactically correct code generation:

```bash
# llama.cpp
./main -m model.gguf --grammar-file lang/tools/constrained/synoema.gbnf \
  -p "-- Quicksort in Synoema" -n 128
```

Full pipeline: [constrained-decoding-e2e.md](constrained-decoding-e2e.md)

---

## Interpreter vs JIT

Both backends support the full language. JIT compiles to native x86-64 via Cranelift.

| Feature | `run` (interpreter) | `jit` (Cranelift) |
|---------|:-------------------:|:-----------------:|
| All language features | Yes | Yes |
| File/Network I/O | Yes | No |
| Concurrency (chan/send/recv) | Yes | No |
| Constant folding / DCE | — | Yes |
| Speed | 1x | ~4.4x |

Use `run` for development and I/O. Use `jit` for performance.

---

## Further Reading

- [Installation Guide](install.md) — all install methods with troubleshooting
- [Formal Language Specification](specs/language_reference.md) — EBNF grammar, type rules, operational semantics
- [LLM Quick Reference](llm/synoema.md) — token-optimized reference for code generation
- [Testing Guide](testing.md) — test infrastructure, running tests
- [Scientific Foundations](research/scientific_foundations.md) — peer-reviewed research behind the design
