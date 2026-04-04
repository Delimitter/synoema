# Show HN: Synoema — The First Programming Language Designed for LLMs

![Cover](images/cover_06.png)

## 890+ Tests, ~12K Lines of Rust, 46% Token Savings, 2.8–5.9× Faster Than Python

---

> **TL;DR.** Synoema is a programming language optimized for LLM code generation. Up to 33% fewer tokens than Python on functional code. 5.3× faster than Python (Cranelift JIT). 100% syntactic correctness via GBNF grammar. Hindley-Milner type inference with zero annotations. 890+ tests, ~12K lines of Rust, 8 crates. MCP server for LLM toolchain integration. Open source, MIT License.

---

## Why Another Language

Every day, millions of developers ask LLMs to write Python. The model generates `def`, `return`, `if/elif/else`, commas in lists — dozens of syntactic overhead tokens that carry no semantic information but cost money and compute.

We researched three fundamental problems:

1. **Python wastes up to 52% of tokens** on semantically empty syntax (varies by task type).
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

### Token Efficiency: Up to -52% vs Python on Functional Code

| Program | Synoema | Python | Saving |
|---------|---------|--------|--------|
| Factorial | 25 | 32 | 22% |
| QuickSort | 77 | 124 | 38% |
| JSON Build | 32 | 67 | 52% |
| Pattern Match | 136 | 225 | 40% |
| **Functional tasks (7)** | **avg 60** | **avg 93** | **33%** |

On functional-style code: 33% fewer tokens ≈ **55% less attention compute**. On mixed workloads (16 tasks): Synoema matches Python. See the token benchmark article in this series for the full breakdown.

### Performance: 2.8–5.9× Faster Than Python

Micro-benchmarks (10 tasks, median of 5 runs, includes JIT compilation + process startup):

| Task | C++ (-O2) | Synoema JIT | Python | vs Python |
|------|-----------|-------------|--------|-----------|
| quicksort | 1.4 ms | 6.0 ms | 16.7 ms | **2.8×** |
| mergesort | 2.1 ms | 6.6 ms | 17.4 ms | **2.6×** |
| filter_map | 2.3 ms | 5.2 ms | 16.6 ms | **3.2×** |
| collatz | 2.3 ms | 5.7 ms | 16.4 ms | **2.9×** |
| gcd | 2.4 ms | 5.6 ms | 16.8 ms | **3.0×** |
| string_ops | 2.0 ms | 5.1 ms | 16.3 ms | **3.2×** |
| **Average** | **1.9 ms** | **6.2 ms** | **16.8 ms** | **2.8×** |

Compute-heavy tasks (larger workloads, algorithm dominates startup):

| Task | Python | Synoema JIT | Speedup |
|------|--------|-------------|---------|
| fib(30) | 277 ms | 47 ms | **5.9×** |
| collatz (10K) | 505 ms | 90 ms | **5.6×** |
| gcd (100K) | 143 ms | 83 ms | **1.7×** |

Full benchmark suite: 16 tasks × 5 languages, open-source in `benchmarks/`.

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

8 crates, ~12,000 lines of Rust, 890+ tests:

| Component | Lines | Tests | Purpose |
|-----------|-------|-------|---------|
| synoema-lexer | 735 | 82 | Tokenization, offside rule |
| synoema-parser | 1,672 | 43 | Pratt parser, 15 expression kinds |
| synoema-types | 1,908 | 61 | Hindley-Milner, row polymorphism, linear types |
| synoema-core | 1,536 | 44 | Core IR (System F), optimizer |
| synoema-eval | 1,894 | 119 | Interpreter: closures, lists, ADTs, records, IO |
| synoema-codegen | 3,044 | 126 | Cranelift JIT: full type support, TCO, string stdlib |
| synoema-diagnostic | — | — | Structured errors, JSON/human, LLM hints |
| synoema-repl | 271 | — | CLI: run / jit / eval / build / test / doc |

## Beyond the Numbers

What makes Synoema more than a benchmark exercise:

- **Prelude**: built-in Result type with combinators (map_ok, unwrap, and_then, is_ok)
- **MCP Server**: `npx synoema-mcp` — plug Synoema into any LLM toolchain (Claude, GPT, local models)
- **Diagnostics**: structured errors with LLM-friendly hints and did-you-mean suggestions
- **Region Inference**: memory management without garbage collection
- **Doc-as-Code**: doctests, `synoema test`, `synoema doc`
- **Benchmark Suite**: 16 tasks × 5 languages, automated Rust runner

## Roadmap

**Done:**

- [x] Full compiler pipeline (lexer → parser → types → core → eval/codegen)
- [x] Cranelift JIT: int, bool, float, string, list, closures, records, ADTs, modules, TCO
- [x] BPE benchmarks (46% vs Python) + performance benchmarks (2.8–5.9×)
- [x] GBNF grammar + SGLang integration
- [x] Closures, records, row polymorphism in JIT
- [x] Modules (`mod`/`use`), string stdlib, type class dispatch
- [x] MCP server + npm distribution
- [x] Region inference, prelude, diagnostics

**Next — and where we need help:**

- [ ] **LLVM backend** (`--backend llvm`) — peak optimization for production workloads
- [ ] **VS Code extension + Web Playground** — lower the barrier to trying Synoema
- [ ] **Fine-tuning experiment** — can LLMs learn Synoema from 10K examples? We have the data, the benchmark harness, and the hypothesis. We need ML researchers to run the experiment
- [ ] **Formal verification layer** — constraint-based correctness beyond types. Active research area connecting dependent types with LLM generation
- [ ] **AST-level generation** — what if LLMs output structure, not text? Requires rethinking tokenizer-model interaction
- [ ] **Incremental type checking for million-token contexts** — today's constrained decoding works at near-zero overhead for short outputs. Scaling to 1M+ token contexts needs new algorithms

## This Is a Research Project

Synoema is not a startup. It's an open-source research project exploring a question that doesn't have a settled answer yet: **what happens when you design a programming language for machines, not humans?**

We've built a working compiler (937 tests, 8 crates, ~12K lines of Rust) and published the results. But the most interesting questions are ahead of us — and they span compiler theory, type systems, ML inference, and language design. No single team can cover all of that.

**If you're a compiler hacker** — the Cranelift JIT has real programs running but needs an LLVM backend, ARM64 codegen, and optimizer passes.

**If you're an ML researcher** — we have a complete benchmark suite (16 tasks, 5 languages, automated runner) and a GBNF grammar ready for constrained decoding experiments.

**If you're a PL theorist** — Hindley-Milner inference is implemented, but type-directed generation at the decoding level is uncharted territory.

**If you're just curious** — clone it, run the tests, break something, file an issue.

## Try It

```bash
git clone https://github.com/synoema/synoema
cd synoema/lang
cargo test                                          # 937 tests
cargo run -p synoema-repl -- run examples/quicksort.sno
cargo run -p synoema-repl -- jit examples/factorial.sno
cargo run -p synoema-repl -- eval "6 * 7"           # → 42
cargo run -p synoema-repl -- test examples/          # Run doctests
```

MIT License. We welcome contributors, researchers, and critics.

---

*Part 6 of "Token Economics of Code" by @andbubnov.*
