# Show HN: Synoema — The First Programming Language Designed for LLMs

## 264 Tests, 7055 Lines of Rust, 46% Token Savings, 4.4× Faster Than Python

---

> **TL;DR.** Synoema is a programming language optimized for LLM code generation. 46% fewer tokens than Python. 4.4× faster than Python (Cranelift JIT). 100% syntactic correctness via GBNF grammar. Hindley-Milner type inference with zero annotations. Open source, MIT License.

---

## Why Another Language

Every day, millions of developers ask LLMs to write Python. The model generates `def`, `return`, `if/elif/else`, commas in lists — dozens of syntactic overhead tokens that carry no semantic information but cost money and compute.

We researched three fundamental problems:

1. **Python wastes 46% of tokens** on semantically empty syntax.
2. **33.6% of LLM code errors are type errors** — formally eliminable.
3. **Interpretation is slow** — generated code can compile to native in milliseconds.

Synoema [sy-NO-e-ma] solves all three. From Greek σύν (together) + νόημα (content of thought) — the language of *shared understanding* between human and LLM.

## Quick Start

```bash
git clone https://github.com/synoema/synoema
cd synoema && cargo build --release

synoema run examples/quicksort.sno   # → [1 2 3 4 5 6 7 8 9]
synoema jit examples/factorial.sno   # → 3628800
synoema eval "6 * 7"                 # → 42
```

## What the Code Looks Like

```
-- Factorial
fac 0 = 1
fac n = n * fac (n - 1)

-- QuickSort
qsort [] = []
qsort (p:xs) = qsort lo ++ [p] ++ qsort hi
  lo = [x | x <- xs , x <= p]
  hi = [x | x <- xs , x > p]

-- Map
map f [] = []
map f (x:xs) = f x : map f xs
```

Syntax: pattern matching, `? cond -> then : else`, comma-free lists `[1 2 3]`, list comprehensions, `where`-blocks via indentation.

## Key Numbers

### Token Efficiency: -46% vs Python

| Program | Synoema | Python | Saving |
|---------|---------|--------|--------|
| Factorial | 16 | 29 | 45% |
| Map | 20 | 42 | 52% |
| QuickSort | 51 | 83 | 39% |
| Filter | 27 | 67 | 60% |
| **Total (12 programs)** | **332** | **615** | **46%** |

Quadratic attention: 46% fewer tokens ≈ **71% less computation**.

### Performance: 4.4× Faster Than Python

| Benchmark | Python | Synoema JIT | Speedup |
|-----------|--------|-------------|---------|
| fib(30) | 277 ms | 47 ms | 5.9× |
| gcd (100K) | 143 ms | 83 ms | 1.7× |
| collatz (10K) | 505 ms | 90 ms | 5.6× |
| **Average** | | | **4.4×** |

### BPE Alignment: 33/33 Operators = 1 BPE Token

Every Synoema operator encodes to exactly 1 BPE token on cl100k_base (GPT-4/Claude) and o200k_base (GPT-4o). Zero bridge tokens. Zero syntactic overhead.

## Constrained Decoding

```python
# SGLang / vLLM / XGrammar
response = client.chat.completions.create(
    model="default",
    messages=[{"role": "user", "content": "Write quicksort in Synoema"}],
    extra_body={"ebnf": open("synoema.gbnf").read()},
)
# Result: 100% syntactically valid code
```

## Architecture

7 crates, 7,055 lines of Rust, 264 tests:

| Component | Lines | Tests | Purpose |
|-----------|-------|-------|---------|
| synoema-lexer | 706 | 80 | Tokenization, offside rule |
| synoema-parser | 1,398 | 36 | Pratt parser, 15 expression kinds |
| synoema-types | 1,453 | 42 | Hindley-Milner, let-polymorphism |
| synoema-core | 969 | 26 | Core IR (System F), desugaring |
| synoema-eval | 1,314 | 46 | Interpreter: closures, lists, ADTs |
| synoema-codegen | 944 | 34 | Cranelift JIT → native x86-64 |
| synoema-repl | 271 | — | CLI: run / jit / eval / REPL |

## Roadmap

- [x] Full compiler pipeline (lexer → parser → types → core → eval/codegen)
- [x] Cranelift JIT (integers + lists + pattern matching)
- [x] BPE benchmarks (46% vs Python) + performance benchmarks (4.4×)
- [x] GBNF grammar + SGLang integration
- [ ] Closures in JIT (map, filter via native code)
- [ ] Records + row polymorphism
- [ ] Modules (`mod`/`use`)
- [ ] LLVM backend (`--backend llvm`)
- [ ] VS Code extension + Web Playground

## Try It

```bash
git clone https://github.com/synoema/synoema
cd synoema
cargo test                                          # 264 tests
cargo run -p synoema-repl -- run examples/quicksort.sno
cargo run -p synoema-repl -- jit examples/factorial.sno
```

MIT License. Contributions welcome.

---

*Sixth article in "Token Economics of Code." Previous: [#1 Token Cost], [#2 BPE Anatomy], [#3 Constrained Decoding], [#4 Compilation], [#5 Hindley-Milner].*
