# Synoema

***The language of shared understanding***

**Synoema** [sy-NO-e-ma] — a BPE-aligned programming language designed for LLM code generation. From Greek σύν (together) + νόημα (content of thought): the optimal interface between human intent and machine intelligence.

Synoema is the first programming language engineered from mathematical first principles to minimize token consumption, eliminate hallucinations through type-guided constrained decoding, and compile to native code via Cranelift JIT.

> **Status: research project.** Synoema is an active research prototype — not yet production-ready. APIs, syntax, and internals may change. Contributions and feedback welcome.

## Why Synoema?

Every token costs money, compute, and latency. Synoema saves **46% of tokens** compared to Python — verified across 12 benchmark programs.

```
Program              Synoema  Python  Saving
─────────────────────────────────────────────
Factorial               16      29     45%
Map                     20      42     52%
QuickSort               51      83     39%
FizzBuzz                44      64     31%
Filter                  27      67     60%
Fibonacci               26      46     44%
Sum List                16      33     52%
Length                  16      30     47%
Reverse                 18      35     49%
Compose & Apply         38      75     49%
Maximum                 28      58     52%
Zip                     32      53     40%
─────────────────────────────────────────────
TOTAL                  332     615     46%
```

Due to quadratic attention cost, **46% fewer tokens ≈ 71% less attention compute.**

And it's not just shorter — it's **faster** (JIT-compiled via Cranelift):

```
Benchmark           Python    Synoema JIT    Speedup
────────────────────────────────────────────────────
fib(30)              277ms       47ms          5.9×
gcd (100K iter)      143ms       83ms          1.7×
collatz (10K)        505ms       90ms          5.6×
────────────────────────────────────────────────────
Average                                        4.4×
```

## Show Me The Code

```synoema
-- Factorial with pattern matching (16 tokens vs Python's 29)
fac 0 = 1
fac n = n * fac (n - 1)

-- QuickSort with list comprehension and where-bindings
qsort [] = []
qsort (p:xs) = qsort lo ++ [p] ++ qsort hi
  lo = [x | x <- xs , x <= p]
  hi = [x | x <- xs , x > p]

-- Higher-order functions, pipes, lambdas
result = [1..10] |> filter (\x -> x % 2 == 0) |> map (\x -> x * x) |> sum

-- Algebraic data types with pattern matching
Maybe a = Just a | None

safe_div x 0 = None
safe_div x y = Just (x / y)

-- Conditional: ? -> : (3 tokens vs if/else's 4-5)
abs x = ? x < 0 -> -x : x

-- FizzBuzz (chained conditionals, no extra syntax)
fizzbuzz n =
  ? n % 15 == 0 -> "FizzBuzz"
  : ? n % 3 == 0 -> "Fizz"
  : ? n % 5 == 0 -> "Buzz"
  : show n

-- Records and field access
point x y = {x = x, y = y}
dist_sq p = p.x * p.x + p.y * p.y

-- Type classes
trait Show a
  show : a -> String

Color = Red | Green | Blue

impl Show Color
  show Red   = "red"
  show Green = "green"
  show Blue  = "blue"

-- Modules: namespace isolation
mod Math
  square x = x * x
  pi = 3.14159

use Math (square pi)

-- IO
main = print "Hello, world!"

-- Type signatures (optional — types are inferred)
map : (a -> b) -> List a -> List b
```

No `def`. No `return`. No commas in lists. No `if`/`else`. Every operator is a single BPE token.

## Getting Started

### Install

```bash
cargo install synoema-repl
synoema jit examples/factorial.sno   # 3628800
synoema run examples/quicksort.sno   # [1 2 3 4 5 6 7 8 9]
```

### Build from source

```bash
git clone https://github.com/Delimitter/synoema
cd synoema/lang
cargo build
```

### Run

```bash
# Interpreter — supports all language features
cargo run -p synoema-repl -- run examples/quicksort.sno

# JIT-compile and run (Cranelift native, 4.4× faster than Python)
cargo run -p synoema-repl -- jit examples/factorial.sno

# Evaluate a single expression
cargo run -p synoema-repl -- eval "6 * 7"

# Interactive REPL
cargo run -p synoema-repl

# Structured error output (JSON for LLM toolchains)
cargo run -p synoema-repl -- --errors json run file.sno
```

### Test

```bash
cargo test        # 634 tests across all crates — all green
```

Full testing guide: [../docs/testing.md](../docs/testing.md)

### Stress Test Dashboard

```bash
cargo run -p synoema-repl -- run examples/stress_server.sno
# Open: http://localhost:8765/stress_tests.html
```

Server guide: [../docs/stress-server.md](../docs/stress-server.md)

### Try the examples

```bash
cargo run -p synoema-repl -- run  examples/quicksort.sno    # [1 2 3 4 5 6 7 8 9]
cargo run -p synoema-repl -- jit  examples/factorial.sno    # 3628800
cargo run -p synoema-repl -- run  examples/fizzbuzz.sno     # FizzBuzz
cargo run -p synoema-repl -- jit  examples/euler1.sno       # 233168
cargo run -p synoema-repl -- run  examples/higher_order.sno # 12
```

## Key Design Principles

1. **BPE-Aligned Grammar** — All 33 operators map to exactly 1 BPE token in cl100k_base/Llama tokenizers. No wasted tokens on syntax.

2. **Hindley-Milner Type Inference** — Full polymorphic type inference without annotations. Types reduce hallucinations by 75% (Mündler et al., PLDI 2025).

3. **Strict Evaluation** — No lazy thunks. Predictable performance like OCaml/Rust, not Haskell.

4. **Deterministic CFG** — Grammar compiles to FSM for zero-overhead constrained decoding (XGrammar/SGLang compatible).

5. **Expression-Oriented** — Everything is an expression. No `return` keyword needed.

## Architecture

```
Source → Lexer → Parser → Type Check → Core IR ─┬→ Interpreter (all features)
  .sno                                  System F  │    Closures, ADTs, strings, IO
        Offside  Pratt   HM infer.               │
        rule     parser  Algorithm W             └→ Cranelift JIT (4.4× faster)
                                                       Native x86-64
                                                       Full feature parity
```

| Crate | Lines | Tests | Purpose |
|-------|------:|------:|---------|
| synoema-lexer | ~800 | 82 | Tokenization + offside rule |
| synoema-parser | ~1700 | 43 | Pratt parsing, 15 ExprKind |
| synoema-types | ~1900 | 61 | Hindley-Milner + row polymorphism |
| synoema-core | ~1600 | 44 | Core IR + desugaring + optimizer |
| synoema-eval | ~1900 | 137 | Tree-walking interpreter |
| synoema-codegen | ~3100 | 191 | Cranelift JIT compiler + runtime |
| synoema-diagnostic | ~400 | — | Structured errors (human + JSON) |
| synoema-repl | ~300 | — | CLI: run / jit / eval / REPL |
| **Total** | **~12000** | **634** | |

## Language Reference

| Feature | Syntax | Note |
|---------|--------|------|
| Function definition | `add x y = x + y` | No `def`, no `return` |
| Pattern matching | `fac 0 = 1` / `fac n = ...` | Multiple equations |
| Lambda | `\x -> x + 1` | `\` is 1 token |
| Conditional | `? cond -> then : else` | 3 tokens vs `if/elif/else` |
| Lists | `[1 2 3]` | No commas |
| List comprehension | `[x \| x <- xs , p x]` | Standard notation |
| Range | `[1..100]` | Inclusive |
| Cons | `x:xs` | Head : tail |
| Pipe | `data \|> f \|> g` | Left-to-right composition |
| Compose | `f >> g` | Right-to-left composition |
| Records | `{x = 3, y = 4}`, `r.x` | Field access |
| Record patterns | `get_x {x = v} = v` | Destructuring |
| Modules | `mod Math` / `use Math (f g)` | Lexical namespaces |
| Algebraic data types | `Shape = Circle r \| Rect w h` | Sum types |
| Type classes | `trait Show a` / `impl Show Color` | Ad-hoc polymorphism |
| IO | `print "hello"` / `readline ()` | Effect primitives |
| Float | `3.14`, `sqrt`, `floor`, `ceil` | FloatNode heap-alloc |
| Power | `2 ** 10` | 1 BPE token |
| Type signature | `f : Int -> Int` | Optional annotation |
| Where-binding | indented below definition | Block scope |

### Interpreter vs JIT

| Capability | `run` (interpreter) | `jit` (Cranelift) |
|------------|:-------------------:|:-----------------:|
| Integers & arithmetic | ✓ | ✓ |
| Booleans | ✓ | ✓ |
| Float (`3.14`, `**`, `sqrt`) | ✓ | ✓ |
| Strings + `show` + `==` | ✓ | ✓ |
| Pattern matching (nested) | ✓ | ✓ |
| Recursion + TCO | ✓ | ✓ |
| Lists + comprehensions | ✓ | ✓ |
| Ranges `[a..b]` | ✓ | ✓ |
| Closures / HOF | ✓ | ✓ |
| `map` / `filter` / `foldl` | ✓ | ✓ |
| Records + field access | ✓ | ✓ |
| Modules (`mod`/`use`) | ✓ | ✓ |
| ADTs + pattern matching | ✓ | ✓ |
| Type classes (`trait`/`impl`) | ✓ | ✓ |
| IO (`print`/`readline`) | ✓ | ✓ |
| Constant folding / DCE | — | ✓ |

## Constrained Decoding (LLM Integration)

Synoema includes a GBNF grammar for constrained decoding. Any LLM can generate **guaranteed syntactically correct** Synoema code:

```python
# SGLang / vLLM / TensorRT-LLM (via XGrammar)
response = client.chat.completions.create(
    model="default",
    messages=[{"role": "user", "content": "Write factorial in Synoema"}],
    extra_body={"ebnf": open("tools/constrained/synoema.gbnf").read()},
)
# Output is 100% syntactically valid Synoema — guaranteed by grammar
```

```bash
# llama.cpp
./main -m model.gguf --grammar-file tools/constrained/synoema.gbnf \
  -p "-- Quicksort in Synoema" -n 128
```

Properties: deterministic CFG, BPE-aligned (zero bridge tokens), compiles to FSM in closed form.

## Structured Diagnostics

Synoema produces LLM-friendly structured errors via the `synoema-diagnostic` crate:

```bash
$ synoema --errors json run broken.sno
{"severity":"error","code":"E0012","message":"type mismatch","span":{"line":3,"col":5},"expected":"Int","got":"String"}
```

Human-readable by default, JSON on `--errors json`. Designed for the generate → check → fix loop.

## Scientific Foundations

Synoema's design is grounded in peer-reviewed research:

- **Token efficiency matters quadratically** — Vaswani et al. (2017), attention cost O(n²·d)
- **Context rot degrades quality** — Hong et al. (2025), Chroma Research
- **Type constraints reduce errors by 75%** — Mündler et al. (2025), PLDI, ACM SIGPLAN
- **Grammar-aligned decoding preserves quality** — Park et al. (2024), NeurIPS
- **BPE misalignment degrades accuracy** — Beurer-Kellner et al. (2024), ICML
- **Deterministic CFG enables zero-overhead constrained decoding** — Tian et al. (2024), CoLM

Full bibliography: [docs/research/scientific_foundations.md](../docs/research/scientific_foundations.md)

## Roadmap

- [x] Lexer, parser, Hindley-Milner type inference
- [x] Tree-walking interpreter with closures, ADTs, strings
- [x] BPE benchmarks — 46% savings vs Python on 12 programs
- [x] Core IR (System F + desugaring transforms + constant folding / DCE)
- [x] Cranelift JIT — native x86-64, 4.4× faster than Python
- [x] Constrained decoding — GBNF for SGLang / XGrammar / llama.cpp
- [x] Closures in JIT — `map`, `filter`, list comprehensions via native code
- [x] Strings in JIT — tagged pointer, `show`, `++`, `==`
- [x] Records in JIT — FNV-hash field lookup, record pattern matching
- [x] Modules — `mod` / `use`, lexical namespacing
- [x] Tail call optimization — iterative eval, 64MB stack thread
- [x] Arena-based memory — 8MB bump allocator, no leaks
- [x] ADTs in JIT — full nested pattern matching, string literal patterns
- [x] Float in JIT — `3.14`, arithmetic, comparisons, `sqrt`/`floor`/`ceil`/`**`
- [x] IO in JIT — `print` / `readline` via FFI
- [x] `show` for all types in JIT — int, bool, float, string, list, ADT, record
- [x] Type class dispatch in JIT — user-defined `impl Show` overrides built-in
- [x] Higher-order stdlib in JIT — `map` / `filter` / `foldl` with closure ABI
- [x] Structured diagnostics — `synoema-diagnostic` crate, JSON + human renderers
- [ ] LSP server — autocomplete, go-to-definition, inline errors
- [ ] Web playground — WASM-compiled interpreter in the browser
- [ ] LLVM backend — `--backend llvm` for maximum optimization

## License

MIT
