# Synoema

***The language of shared understanding***

**Synoema** [sy-NO-e-ma] — a BPE-aligned programming language designed for LLM code generation. From Greek σύν (together) + νόημα (content of thought): the optimal interface between human intent and machine intelligence.

Synoema is the first programming language engineered from mathematical first principles to minimize token consumption, eliminate hallucinations through type-guided constrained decoding, and compile to native code via Cranelift JIT.

## Why Synoema?

Every token costs money, compute, and latency. Synoema saves **46% of tokens** compared to Python — verified across 12 benchmark programs.

```
Program              Synoema  Python  Saving
─────────────────────────────────────────
Factorial               16      29   45%
Map                     20      42   52%
QuickSort               51      83   39%
FizzBuzz                44      64   31%
Filter                  27      67   60%
Fibonacci               26      46   44%
Sum List                16      33   52%
Length                   16      30   47%
Reverse                 18      35   49%
Compose & Apply         38      75   49%
Maximum                 28      58   52%
Zip                     32      53   40%
─────────────────────────────────────────
TOTAL                  332     615   46%
```

Due to quadratic attention cost, **46% fewer tokens ≈ 71% less attention compute.**

And it's not just shorter — it's **faster** (JIT-compiled via Cranelift):

```
Benchmark           Python    Synoema JIT    Speedup
────────────────────────────────────────────────────
fib(30)              277ms       47ms       5.9x
gcd (100K iter)      143ms       83ms       1.7x
collatz (10K)        505ms       90ms       5.6x
────────────────────────────────────────────────────
Average                                    4.4x
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
result = [1 2 3 4 5] |> filter even |> map (\x -> x * 2) |> sum

-- Algebraic data types
Maybe a = Just a | None

-- Conditional: ? -> : (3 tokens vs if/else's 4-5)
abs x = ? x < 0 -> -x : x

-- Type signatures (optional — types are inferred)
map : (a -> b) -> List a -> List b
```

No `def`. No `return`. No commas in lists. No `if`/`else`. Every operator is a single BPE token.

## Getting Started

```bash
# Build
cargo build --release -p synoema-repl

# Run a program (interpreter — supports all features)
cargo run -p synoema-repl -- run examples/quicksort.sno

# JIT-compile and run (Cranelift native — 4.4x faster than Python)
cargo run -p synoema-repl -- jit examples/factorial.sno

# Evaluate an expression
cargo run -p synoema-repl -- eval "42 + 1"

# Interactive REPL
cargo run -p synoema-repl

# Run tests (248 tests)
cargo test
```

## Key Design Principles

1. **BPE-Aligned Grammar** — All 33 operators map to exactly 1 BPE token in cl100k_base/Llama tokenizers. No wasted tokens on syntax.

2. **Hindley-Milner Type Inference** — Full polymorphic type inference without annotations. Types reduce hallucinations by 75% (Mündler et al., PLDI 2025).

3. **Strict Evaluation** — No lazy thunks. Predictable performance like OCaml/Rust, not Haskell.

4. **Deterministic CFG** — Grammar compiles to FSM for zero-overhead constrained decoding (XGrammar/SGLang compatible).

5. **Expression-Oriented** — Everything is an expression. No `return` keyword needed.

## Architecture

```
Source → Lexer → Parser → Type Check → Core IR ─┬→ Interpreter (full features)
  .sno     ↓        ↓         ↓           ↓      │    Lists, closures, ADTs
        Tokens    AST     Typed AST    System F  │
        (offside        (HM inference)           └→ Cranelift JIT (4.4x faster)
         rule)                                        Native x86-64, integers
                                                      Pattern match, recursion
```

| Crate | Lines | Tests | Purpose |
|-------|-------|-------|---------|
| synoema-lexer | 706 | 80 | Tokenization + offside rule |
| synoema-parser | 1398 | 36 | Pratt parsing, 13 precedence levels |
| synoema-types | 1453 | 42 | Hindley-Milner type inference |
| synoema-core | 969 | 26 | Core IR + desugaring (System F) |
| synoema-eval | 1314 | 46 | Tree-walking interpreter |
| synoema-codegen | 472 | 18 | Cranelift JIT compiler |
| synoema-repl | 271 | — | CLI: run / jit / eval / REPL |
| **Total** | **6583** | **248** | |

## Language Features

| Feature | Syntax | Tokens |
|---------|--------|--------|
| Function def | `add x y = x + y` | No `def`, no `return` |
| Pattern match | `fac 0 = 1` | Multiple equations |
| Lambda | `\x -> x + 1` | `\` = 1 token |
| Conditional | `? cond -> then : else` | 3 tokens vs 4-5 |
| Lists | `[1 2 3]` | No commas |
| List comp | `[x \| x <- xs , p x]` | Standard syntax |
| Range | `[1..100]` | Two dots |
| Pipe | `data \|> f \|> g` | Left-to-right flow |
| Compose | `f >> g` | Function composition |
| ADT | `Maybe a = Just a \| None` | Clean algebraic types |
| Type sig | `f : Int -> Int` | Optional annotation |
| Block | `a = 10` (indented) | Where-style bindings |

## Constrained Decoding (LLM Integration)

Synoema includes a GBNF grammar file for constrained decoding. Any LLM can generate **guaranteed syntactically correct** Synoema code:

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

## Scientific Foundations

Synoema's design is grounded in peer-reviewed research:

- **Token efficiency matters quadratically** — Vaswani et al. (2017), attention cost O(n²·d)
- **Context rot degrades quality** — Hong et al. (2025), Chroma Research
- **Type constraints reduce errors by 75%** — Mündler et al. (2025), PLDI, ACM SIGPLAN
- **Grammar-aligned decoding preserves quality** — Park et al. (2024), NeurIPS
- **BPE misalignment degrades accuracy** — Beurer-Kellner et al. (2024), ICML
- **Deterministic CFG enables zero-overhead constrained decoding** — Tian et al. (2024), CoLM

Full bibliography: [Synoema Scientific Foundations](docs/scientific_foundations.md)

## Roadmap

- [x] **Milestone 1: Working Language** — Lexer, parser, type inference, interpreter, REPL
- [x] **Phase 5: BPE Benchmarks** — 46% savings vs Python verified on 12 programs
- [x] **Phase 6: Core IR** — Desugaring to System F
- [x] **Phase 7: Cranelift JIT** — Native compilation, 4.4x faster than Python
- [x] **Phase 8: Constrained Decoding** — GBNF grammar for SGLang/XGrammar/llama.cpp
- [ ] **Phase 9: Language Extensions** — Records, modules, IO, FFI, lists in JIT
- [ ] **Phase 10: Optimization** — Region-based memory, whole-program optimization, LLVM backend

## Contributing

Synoema is in active development. We're looking for contributors interested in:

- Compiler engineering (Rust, LLVM)
- Programming language theory (type systems, formal semantics)
- LLM infrastructure (constrained decoding, tokenization)
- Benchmarking and evaluation

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT
