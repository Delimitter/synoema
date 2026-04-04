# Type-Guided Constrained Decoding: How to Stop LLMs from Hallucinating Code

![Cover](images/cover_03.png)

## From GBNF Grammars to Type-Directed Generation: Guarantees Instead of Hope

---

> **Who this is for.** If you've ever had ChatGPT generate code that doesn't compile — this article explains how to eliminate that completely. All technical terms explained in footnotes and the glossary at the end.

---

In previous articles, we showed that reducing tokens saves money, energy, and compute. But there's a more serious problem: LLMs generate **incorrect** code. And every retry doubles the token spend.

## The Scale of the Problem

Type errors account for 33.6% of all failures in LLM-generated code (Mündler et al., PLDI 2025[^pldi]). These aren't typos — they're structural errors: wrong argument types, incompatible return values, accessing nonexistent fields.

When an LLM generates a sort function that doesn't compile, the cost doubles — either a human fixes it (time) or an agent retries (tokens).

But what if the model **physically cannot** generate syntactically invalid code?

## Three Levels of Constraints

### Level 1: Grammar (Syntactic Correctness)

At each generation step, the set of grammatically[^grammar] valid tokens is determined. All others are masked — probability set to zero.

Example: if the model just generated `[`, then the next token can be a number, identifier, `]`, or `[` — but not `+`, not `=`, not `)`.

**Tools:**

- **XGrammar[^xgrammar]** — default backend in SGLang[^sglang], vLLM, TensorRT-LLM[^trt]. Works with context-free grammars (CFG[^cfg]). Approaches zero overhead.
- **Outlines[^outlines]** — structured generation via finite state machines (FSM[^fsm]). Supports regex and CFG.
- **llama.cpp[^llamacpp]** — built-in GBNF grammar[^gbnf] support.
- **Guidance** (Microsoft) — template-based generation with constraints.

Result: **100% syntactic correctness.** Every generated fragment is a valid program.

### Level 2: Types (Semantic Correctness)

Grammar guarantees that `f x = x + 1` is syntactically valid. But not that `x` is a number. Type-constrained decoding[^tcd] adds a second layer: only tokens compatible with the current type context are allowed.

Mündler et al. (PLDI 2025) showed that type-constrained decoding reduces compilation errors by **74.8%** compared to 9.0% for syntax-only constraints.

This requires type inference[^typeinf] — so the compiler can determine valid types at every generation point without explicit annotations.

### Level 3: Specification (Logical Correctness)

The most powerful level: constraints based on formal specification. A sort function doesn't just have the right type — it actually sorts. This is an area of active research (dependent types, refinement types). Not yet in production tools.

## How XGrammar Works

XGrammar's key optimization: **splitting the vocabulary into two classes:**

**Context-independent tokens (~80%+).** Validity determined at preprocessing, before generation. For each grammar state, a bitmask of valid tokens is precomputed. O(1) per token.

**Context-dependent tokens (~20%).** Validity depends on the current PDA[^pda] state. Checked at runtime, but few in number.

Result: **near-zero overhead.** Constrained decoding adds no measurable overhead to TPOT[^tpot].

## BPE Misalignment Breaks Constrained Decoding

This is where language design becomes critical.

When a language grammar isn't aligned to BPE boundaries, constrained decoding faces the **bridge token** problem — a BPE token spanning two grammatical symbols.

**Domino** (ICML 2024[^icml]) showed that bridge tokens distort the model's probability distribution. **Grammar-Aligned Decoding** (NeurIPS 2024[^neurips]) formalized the problem and proposed a fix — but with added overhead.

If a language is designed so bridge tokens **never arise** — every grammatical symbol coincides with one BPE token — the problem disappears entirely.

## Deterministic CFG = Zero Overhead

**Nondeterministic CFG** — when parsing, multiple rules may apply. Requires backtracking[^bt]. Expensive.

**Deterministic CFG (DCFG)[^dcfg]** — exactly one rule applies at each step. Compiles to an FSM. No backtracking. No ambiguity.

Tian et al. (CoLM 2024[^colm]) proved that for DCFGs, constrained decoding compiles in **closed form** — overhead approaches zero.

If a language has a DCFG grammar with BPE-aligned operators, constrained decoding is **free**: zero overhead + zero bridge tokens.

## In Practice: GBNF Grammar

```gbnf
root    ::= program
program ::= (decl newline)* decl newline?
decl    ::= func-def | type-sig | type-def

func-def ::= lower-id ws (pattern ws)* "=" ws expr
cond-expr ::= "?" ws expr ws "->" ws expr ws ":" ws expr
```

Plugging into SGLang:

```python
response = client.chat.completions.create(
    model="default",
    messages=[{"role": "user", "content": "Write factorial"}],
    extra_body={"ebnf": open("synoema.gbnf").read()},
)
# Result: GUARANTEED syntactically valid code
```

Or llama.cpp:

```bash
./main -m model.gguf --grammar-file synoema.gbnf \
  -p "-- Quicksort:" -n 128 --temp 0.2
```

## The Economic Impact

| Lever | Mechanism | Savings |
|-------|-----------|---------|
| BPE-aligned grammar | 46% fewer tokens | -46% direct |
| Quadratic attention | 54% length → 29% cost | -71% on attention |
| Constrained decoding | 0 invalid code → 0 retries | -10–30% |
| Type constraints | -74.8% type errors | -5–15% additional |

Combined: **50–70%** savings in cost and energy vs unoptimized Python generation.

## What's Next

In the next article, we'll introduce **Synoema** — a language with all three levers: BPE-aligned grammar (33 single-token operators), Hindley-Milner[^hm] type inference, and Cranelift[^cranelift] JIT for native speed.

---

*Third article in "Token Economics of Code." Sources: XGrammar (mlc-ai/xgrammar), Domino (ICML 2024), Grammar-Aligned Decoding (NeurIPS 2024), Mündler et al. (PLDI 2025), Tian et al. (CoLM 2024).*

---

## Footnotes

[^pldi]: **PLDI (Programming Language Design and Implementation)** — one of the most prestigious academic conferences on programming languages. Papers undergo rigorous peer review. If a result is published at PLDI, it's trustworthy.

[^grammar]: **Formal grammar** — a set of rules describing which sequences of symbols are valid in a language. For example, "after `[`, the next symbol can be a number, identifier, or `]`" is part of a grammar. Python has a complex grammar (hundreds of rules); JSON has a simple one (~10 rules).

[^xgrammar]: **XGrammar** — constrained decoding engine from the MLC-AI team. The de facto standard for LLM inference. Its key innovation: splitting the vocabulary into "easy" tokens (80%+, checked at preprocessing) and "hard" tokens (20%, checked at runtime), yielding near-zero overhead.

[^sglang]: **SGLang** — open-source LLM inference engine from UC Berkeley. One of the fastest ways to serve LLMs. Supports constrained decoding via XGrammar out of the box.

[^trt]: **TensorRT-LLM** — NVIDIA's inference engine, optimized for their GPUs. Fastest on NVIDIA hardware, but complex to set up.

[^cfg]: **CFG (Context-Free Grammar)** — a class of grammars where each rule has the form "symbol → sequence of symbols." Most programming languages are described by CFGs. JSON, XML, HTML, Python, JavaScript all have CFGs.

[^fsm]: **FSM (Finite State Machine)** — a mathematical model that at any moment is in one of a finite number of states and transitions between them on input symbols. Used for fast checking of whether the next token is valid.

[^outlines]: **Outlines** — open-source library for structured generation. Compiles a grammar or regex into a finite state machine that filters tokens on the fly.

[^llamacpp]: **llama.cpp** — the most popular open-source project for running LLMs on commodity hardware (CPU, Mac M1/M2, budget GPUs). Written in C++, works without Python. Supports GBNF grammars for constrained decoding.

[^gbnf]: **GBNF (GGML BNF)** — grammar description format used in llama.cpp. Extension of standard Backus-Naur Form. Example: `expr ::= number | expr "+" expr`.

[^tcd]: **Type-constrained decoding** — an extension of constrained decoding that checks types in addition to grammar. If a function expects `Int`, the model can't substitute `String`. Requires type inference — automatic type deduction by the compiler.

[^typeinf]: **Type inference** — the compiler's ability to determine types of all expressions automatically, without programmer annotations. Instead of `int add(int x, int y)` (as in C), you write just `add x y = x + y`, and the compiler deduces `Int → Int → Int`. The most powerful type inference algorithm is Hindley-Milner, used in Haskell and ML.

[^pda]: **PDA (Pushdown Automaton)** — an extension of a finite state machine with a stack. Needed for grammars with nesting (brackets, code blocks). A regular FSM can't count brackets — a PDA can.

[^tpot]: **TPOT (Time Per Output Token)** — the time to generate one output token. The main metric for LLM inference speed. For GPT-4: ~20–30 ms; for small models on powerful GPUs: 5–10 ms.

[^icml]: **ICML (International Conference on Machine Learning)** — one of the top three ML conferences (with NeurIPS and ICLR). Publication at ICML signals high-quality research.

[^neurips]: **NeurIPS (Neural Information Processing Systems)** — the largest AI/ML conference. ~15,000 attendees annually. Publication at NeurIPS is the gold standard for ML research.

[^bt]: **Backtracking** — a parsing method where the parser tries one rule, and if it fails, backtracks and tries another. Slow because the same text may be parsed multiple times.

[^dcfg]: **DCFG (Deterministic Context-Free Grammar)** — a subclass of CFG where parsing is unambiguous at every step. Compiles to an efficient automaton. Most real programming languages are DCFGs (or close). Python technically isn't due to indentation, but with the offside rule it approximates one.

[^colm]: **CoLM** — a newer conference at the intersection of language models and formal methods. Focuses on how compiler theory can improve LLMs.

[^hm]: **Hindley-Milner** — the most powerful automatic type inference algorithm, developed in the 1960s–80s. Allows the compiler to determine types of all expressions **without a single annotation**. Used in Haskell, OCaml, F#, Elm. Detailed in the fifth article.

[^cranelift]: **Cranelift** — a compiler backend written in Rust. Converts intermediate representation (IR) to native machine code (x86-64, ARM). Alternative to LLVM: compiles 10× faster, though generated code is ~14% slower. Ideal for JIT compilation where compilation speed matters more than peak optimization.

## Glossary

| Term | Explanation |
|------|-----------|
| **Constrained decoding** | Technology forbidding invalid tokens during generation. Guarantees correctness |
| **XGrammar** | Constrained decoding engine, de facto standard for LLM inference |
| **SGLang** | Open-source LLM inference engine from UC Berkeley |
| **vLLM** | Open-source LLM inference engine with memory optimization |
| **TensorRT-LLM** | NVIDIA's inference engine, fastest on their GPUs |
| **GBNF** | Grammar description format for llama.cpp |
| **llama.cpp** | Popular project for running LLMs on commodity hardware |
| **CFG** | Context-Free Grammar — formal grammar describing language syntax |
| **DCFG** | Deterministic CFG — unambiguous parsing, enables zero-overhead constraints |
| **FSM** | Finite State Machine — model for fast token validity checking |
| **PDA** | Pushdown Automaton — FSM with a stack for nested structures |
| **TPOT** | Time Per Output Token — main LLM inference speed metric |
| **Bridge token** | BPE token spanning two grammar symbol boundaries |
| **Type inference** | Automatic type deduction by the compiler, no annotations needed |
| **Hindley-Milner** | Most powerful type inference algorithm. Used in Haskell, OCaml |
| **Cranelift** | Rust-based compiler backend. Fast JIT compilation to native code |
| **PLDI / ICML / NeurIPS** | Top academic conferences on PL, ML, and AI respectively |
| **Backtracking** | Parsing by trial-and-error with rollbacks. Slow but universal |
