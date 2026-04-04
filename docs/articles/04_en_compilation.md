# Compilation for LLMs: Why a Language for Models Needs Native Code

![Cover](images/cover_04.png)

## Cranelift JIT, 2.8–5.9× Faster Than Python, and Why It Matters for AI Agents

---

> **Who this is for.** If you're building AI agents that generate and execute code, or want to understand why compiled LLM output isn't science fiction but working technology — read on. All terms explained in footnotes and glossary.

---

In previous articles, we showed how to cut tokens by 46% and guarantee syntactic correctness. But there's a third problem: generated code must not only be short and correct — it must be **fast**.

## Context: LLM Agents Write and Run Code

Claude Code, Cursor, Devin, OpenAI Codex — these tools don't just generate code. They **execute** it: run tests, process data, call APIs. The cycle "generate → run → analyze result → repeat" is the foundation of agentic workflows[^agentic].

The problem: almost all agents generate **Python**. And Python is interpreted[^interpreted].

This means: every run goes through the CPython interpreter (slow, single-threaded), no code optimization (Python doesn't know types until runtime via duck typing[^duck]), and serious computation requires C-based libraries (NumPy, pandas).

## The Solution: JIT Compilation

What if LLM-generated code **compiles to native machine code** in milliseconds and runs at C speed?

That's exactly what JIT compilation[^jit] does:

```
LLM generates code (.sno)
    ↓
Parser → AST[^ast] → Type Check → Core IR[^coreir]
    ↓
Cranelift JIT → native x86-64 machine code
    ↓
Execution at C/Rust speed (no interpreter)
```

The entire cycle — from text to native code — takes **< 100 ms**.

## Why Cranelift, Not LLVM

For JIT compilation we need a compiler backend[^backend] — a tool that converts Core IR to machine code. Two main options:

**LLVM[^llvm]** — the industry standard. Used in Clang (C/C++), Rust, Swift, Julia. Generates very fast code but compiles **slowly**: tens of milliseconds per function. Written in C++, pulls gigabytes of dependencies.

**Cranelift[^cranelift]** — written in pure Rust. Compiles **10× faster** than LLVM. Generates code ~86% the quality of LLVM. Ideal for JIT: compilation speed matters more than peak optimization.

| Criterion | LLVM | Cranelift |
|-----------|------|-----------|
| Language | C++ | Rust |
| Compilation speed | 1× | **10×** |
| Code quality | 100% | ~86% |
| Dependencies | Gigabytes | `cargo build` |
| Ideal for | AOT compilation | **JIT compilation** |

## Benchmarks: Synoema JIT vs Python vs C++

We ran a benchmark suite of 12 algorithmic tasks across three languages: CPython 3.12, Synoema JIT (Cranelift, release build), and C++ (`-O2 -std=c++17`). Methodology: median of 5 measured runs, 3 warm-up runs discarded. All times include process startup; Synoema times include JIT compilation.

### Full Suite (10 tasks with all three languages)

| Task | C++ | Synoema JIT | Python | Synoema vs Python |
|------|-----|-------------|--------|-------------------|
| quicksort | 1.4 ms | 6.0 ms | 16.7 ms | **2.8×** |
| mergesort | 2.1 ms | 6.6 ms | 17.4 ms | **2.6×** |
| binary_search | 2.1 ms | 7.4 ms | 16.7 ms | **2.3×** |
| tree_traverse | 1.5 ms | 6.5 ms | 17.0 ms | **2.6×** |
| filter_map | 2.3 ms | 5.2 ms | 16.6 ms | **3.2×** |
| collatz | 2.3 ms | 5.7 ms | 16.4 ms | **2.9×** |
| gcd | 2.4 ms | 5.6 ms | 16.8 ms | **3.0×** |
| fizzbuzz | 1.7 ms | 5.7 ms | 16.8 ms | **3.0×** |
| matrix_mult | 1.5 ms | 8.4 ms | 17.6 ms | **2.1×** |
| string_ops | 2.0 ms | 5.1 ms | 16.3 ms | **3.2×** |
| **Average** | **1.9 ms** | **6.2 ms** | **16.8 ms** | **2.8×** |

### Compute-Heavy Tasks (larger workloads)

For compute-intensive benchmarks where algorithm runtime dominates startup overhead, the speedup is higher:

| Task | Python | Synoema JIT | Speedup |
|------|--------|-------------|---------|
| fib(30) — recursive Fibonacci | 277 ms | 47 ms | **5.9×** |
| collatz (10K numbers) — Collatz conjecture | 505 ms | 90 ms | **5.6×** |
| gcd (100K iterations) — Euclidean algorithm | 143 ms | 83 ms | **1.7×** |
| **Average** | | | **4.4×** |

### What the Numbers Mean

**Micro-benchmarks (small inputs):** 2.1–3.2× faster. Process startup (~5 ms for Synoema, ~16 ms for Python) dominates total time, compressing the ratio.

**Compute-heavy tasks:** up to 5.9× faster. With larger workloads, JIT-compiled native code pulls ahead as startup cost amortizes.

**C++ context:** C++ runs 3× faster than Synoema JIT on average. This is expected — Cranelift generates ~86% quality code vs LLVM (-O2), and Synoema's time includes JIT compilation overhead. The trade-off: Synoema compiles in < 100 ms (no build step), while C++ requires a toolchain.

All benchmarks: 16 tasks × 5 languages, open-source in `benchmarks/`.

## Architecture Pipeline

```
Source code (.sno)
  │
  ├─ Lexer (735 lines Rust, 82 tests)
  │   Tokenization + offside rule[^offside]
  │
  ├─ Parser (1,672 lines, 43 tests)
  │   Pratt parser[^pratt] → AST
  │
  ├��� Type Checker (1,908 lines, 61 tests)
  │   Hindley-Milner inference + row polymorphism → Typed AST
  │
  ├─ Core IR (1,536 lines, 44 tests)
  │   Desugaring + optimizer → System F
  │
  ├─ Diagnostics (synoema-diagnostic)
  │   Structured errors, JSON/human output, LLM hints
  │
  └─ Backend (choice)
      ├─ Interpreter (1,894 lines, 119 tests)
      │   Closures, lists, ADTs, records, modules, IO
      ��
      └─ Cranelift JIT (3,044 lines, 126 tests)
          Native x86-64: int, bool, float, string, list,
          closures, records, ADTs, modules, TCO
```

8 crates[^crate], ~12,000 lines of Rust, 890+ tests, 0 errors.

## What This Means for AI Agents

**With Python:** LLM generates script (200 tokens, 1.5s) → subprocess → Python processes (12s) → total ~15 seconds.

**With Synoema:** LLM generates sno code (108 tokens, 0.8s) → JIT (50ms) → native processing (3s) → total ~4 seconds.

Savings: **73% time**, **46% tokens**, **zero dependencies**.

## What's Changed Since We Started

The project has grown significantly since initial benchmarks:

- **890+ tests** (from 264), all passing, 0 warnings
- **JIT now supports**: closures, records, record update, ADTs, pattern matching, modules, self-recursive TCO, full string stdlib, float arithmetic, type class dispatch
- **Prelude**: built-in Result type with combinators (map_ok, unwrap, and_then)
- **MCP server**: `npx synoema-mcp` integrates Synoema into LLM toolchains
- **Region inference**: memory management without garbage collection
- **Diagnostics**: structured errors with LLM-friendly hints, did-you-mean suggestions

The JIT is no longer a proof of concept — it's a functional compiler backend handling real programs.

## What's Next

Next in the series: **Hindley-Milner** — the type inference system that delivers 100% type safety with zero annotations. This is what makes type-guided constrained decoding possible.

---

*Part 4 of "Token Economics of Code" by @andbubnov. Benchmarks on macOS x86-64, CPython 3.12 vs Synoema v0.1.0-alpha.1 (Cranelift, --release).*

---

## Footnotes

[^agentic]: **Agentic workflows** — an approach where an LLM acts as an autonomous "agent": receives a task, breaks it into steps, writes code, runs it, analyzes the result, and adjusts. Unlike a simple chatbot, an agent can handle multi-step tasks independently.

[^interpreted]: **Interpreted language** — a language whose code is executed "line by line" by an interpreter, without prior compilation to machine code. Python, JavaScript, Ruby are interpreted. C, Rust, Go are compiled (code is first converted to machine instructions, then executed directly by the processor). Interpreted languages are simpler but 10–100× slower than compiled ones.

[^duck]: **Duck typing** — Python's principle: "if it walks like a duck and quacks like a duck, it's a duck." Variable types aren't declared or checked in advance — type errors are discovered only at runtime. Convenient for prototypes, dangerous for production, and makes compilation to efficient machine code impossible.

[^jit]: **JIT (Just-In-Time) compilation** — compiling code to machine instructions immediately before execution, "on the fly." Unlike AOT compilation (Ahead-Of-Time, as in C/Rust) where code is compiled in advance, JIT compiles at launch time. Advantage: no separate build step. LLM generates code → JIT compiles in milliseconds → native execution speed.

[^ast]: **AST (Abstract Syntax Tree)** — a data structure representing a program as a tree. The parser converts code text into an AST, after which the compiler works with the tree, not text. For example, `2 + 3 * 4` becomes a tree where `*` is a child node of `+`.

[^coreir]: **Core IR (Intermediate Representation)** — a simplified "internal language" of the compiler. Source code is first converted to Core IR (syntactic sugar removed, pattern matching expanded to case expressions), then Core IR is compiled to machine code. This is System F — a formalism from type theory.

[^backend]: **Compiler backend** — the part of a compiler that generates final machine code. The frontend (parser, type checker) works with source code; the backend works with machine instructions. LLVM and Cranelift are the two most popular backends.

[^llvm]: **LLVM (Low Level Virtual Machine)** — a compiler framework started in 2003 by Chris Lattner. The de facto standard: Clang (C/C++), Rust, Swift, Julia, Zig are all built on it. Generates maximally optimized machine code but is complex and slow to compile.

[^cranelift]: **Cranelift** — compiler backend created by Bytecode Alliance (Mozilla, Fastly, Intel). Written entirely in Rust. Used in Wasmtime (WebAssembly runtime) and as a JIT backend for several languages. Targets x86-64, ARM64, RISC-V.

[^offside]: **Offside rule** — a principle where code structure is determined by indentation rather than brackets. Like Python: a code block is lines with the same indent level.

[^pratt]: **Pratt parser** �� an expression parsing algorithm invented by Vaughan Pratt in 1973. Elegantly handles operator precedence (multiplication before addition) without recursive descent. Used in GCC, V8 (JavaScript), rustc.

[^crate]: **Crate** — a compilation unit and library in Rust. Analogous to a "package" or "module" in other languages.

## Glossary

| Term | Explanation |
|------|-----------|
| **JIT compilation** | "On-the-fly" compilation — code becomes machine instructions right before execution |
| **AOT compilation** | Compile in advance — as in C/Rust. Slower but maximum optimization |
| **Cranelift** | Rust-based compiler backend. 10× faster compilation than LLVM, ~86% code quality |
| **LLVM** | Industry-standard compiler framework. Used in Clang, Rust, Swift |
| **Backend** | Part of a compiler that generates machine code |
| **AST** | Abstract Syntax Tree — data structure representing a program |
| **Core IR** | Intermediate Representation — simplified internal compiler language |
| **Agentic workflow** | LLM acts as autonomous agent: plans, codes, executes |
| **Duck typing** | Python's principle: types aren't declared, errors found only at runtime |
| **Offside rule** | Code structure via indentation (like Python), not brackets |
| **Pratt parser** | Expression parsing algorithm handling operator precedence |
| **Crate** | Compilation unit in Rust, analogous to a library/package |
