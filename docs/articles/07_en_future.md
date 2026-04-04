# The Future of Code Generation: From Prompts to Compilation

![Cover](images/cover_07.png)

## What Happens When LLMs Become Part of the Compiler

---

> **Who this is for.** If you're curious about where the intersection of AI and programming languages is heading — this is a speculative but grounded picture of the next 3–5 years.

---

This series traveled from the problem (tokens are expensive) through research (BPE misalignment, constrained decoding, type inference) to a concrete solution (Synoema). This final article looks ahead.

## Current Paradigm: LLM as Text Generator

Today, LLMs generate code as **text**:

```
Prompt → LLM → character string → parser → AST → compiler → execution
```

The model doesn't "understand" program structure. It predicts the next token based on statistical patterns. Brackets close not because the model knows grammar rules, but because `)` frequently follows `(` in training data.

This works remarkably well — but creates fundamental limitations: no correctness guarantees, redundancy (text carries information the compiler can infer), and fragility (one missing bracket breaks everything).

## Next Paradigm: LLM as Compiler Component

What if the LLM doesn't generate text, but **directly creates structural program representation**?

```
Prompt → LLM + Grammar Constraints + Type Constraints
       → guaranteed-valid AST
       → JIT compilation
       → native execution
```

This isn't theory — the components already exist:

- **Constrained decoding** (XGrammar, Outlines) guarantees syntactic correctness.
- **Type-guided generation** (PLDI 2025) adds type guarantees.
- **JIT compilation** (Cranelift) converts the result to native code in milliseconds.

Synoema is the first language designed for all three simultaneously. But this is just the beginning.

## Five Open Questions

### 1. Can LLMs Learn Synoema Better Than Python?

Intuition says "no" — Python dominates training data. But the counterargument: Synoema is **simpler**. 7 keywords instead of 35. 33 operators, each one token. Deterministic grammar with no ambiguity.

Hypothesis: fine-tuning[^finetune] on 10K Synoema examples may yield generation quality comparable to Python, with 46% fewer tokens. This is a testable experiment.

### 2. Do We Need AST-Level Generation?

Today LLMs generate *text* that's then parsed into an AST. What if the model directly generates AST nodes?

```
Instead of: "fac 0 = 1" (5 text tokens)
Model generates: FuncDef("fac", [PatLit(0)], Lit(1))  (3 structural tokens)
```

Potentially more efficient: fewer tokens, syntactic errors impossible. But requires rethinking LLM architecture.

### 3. How Do Constraints Scale to 1M+ Token Contexts?

XGrammar works at near-zero overhead for short generations. But at million-token contexts (already supported by Gemini and Claude), PDA state costs grow. New algorithms for incremental constraint checking[^incremental] are needed.

### 4. Can One Language Be Optimal for Both LLMs and Humans?

Synoema is BPE-optimized. But reading `? x > 0 -> x : -x` is less familiar than `if x > 0: return x else: return -x`.

Possible solution: **two representations** of the same program. LLM works with compact BPE-optimized form. Human sees "expanded" representation with familiar syntax. Transformation between them is a bijection[^bijection], performed automatically.

### 5. What Role for Formal Verification?

Constrained decoding guarantees syntax. Type inference guarantees types. But neither guarantees the program **does what it should**.

The next frontier: integrating formal verification[^verification] into the inference pipeline — LLM generates code, and the constraint system checks not just types, but specification (preconditions, postconditions, invariants).

## Vision: Agentic Computation Pipeline

Putting it all together:

```
Human intent (natural language)
    ↓
LLM + Grammar Constraints + Type Constraints
    ↓
Guaranteed-correct code (Synoema)
    ↓
JIT compilation (Cranelift / LLVM)
    ↓
Native execution
    ↓
Result → back to LLM for analysis
```

The entire cycle — **< 1 second**. No pip install. No Docker. No CI/CD. LLM thinks → Synoema formalizes → Cranelift executes → result.

This isn't theory. Synoema already has an MCP server (`npx synoema-mcp`) that plugs into LLM toolchains — Claude, GPT, local models. The agentic loop exists today: LLM requests → Synoema compiles → result returns to LLM. What's missing is the constraint layer in production inference engines.

This isn't replacing programmers. It's a new tool — like the calculator didn't replace mathematicians, but changed which problems became solvable.

## What We've Already Built

Some items from this "future" article are no longer speculative — they're shipped:

- **Closures in JIT** ✅ — map, filter, foldl compile to native code
- **Records + row polymorphism** ✅ — structural typing with zero overhead
- **Modules** ✅ — `mod`/`use` for multi-file programs
- **MCP integration** ✅ — LLM toolchain integration via Model Context Protocol
- **Region inference** ✅ — memory management without GC
- **890+ tests** ✅ — from 264 at launch

The remaining open questions (AST-level generation, fine-tuning experiments, formal verification integration) are active research directions — and this is where the project needs the most help.

## Where We Are

Synoema is an early-stage research project. We have a working compiler (937 tests, 8 crates, ~12K lines of Rust), published benchmarks, and a clear hypothesis. What we don't have is all the answers.

Some things we've proven:
- BPE-aligned grammar saves 46% tokens vs Python — **verified on 16 benchmarks**
- Cranelift JIT runs 2.8–5.9× faster than CPython — **measured, reproducible**
- Hindley-Milner infers all types with zero annotations — **implemented, 61 tests**
- GBNF constrained decoding guarantees 100% syntactic correctness — **working with SGLang/llama.cpp**

Some things we haven't:
- Can LLMs learn Synoema well enough to match Python generation quality? (We have the benchmark harness — need ML researchers to run the experiment)
- Does type-directed generation at decoding time work in practice at scale? (Theory says yes. No one has built it end-to-end)
- What does AST-level generation actually look like? (Requires rethinking the tokenizer-model boundary)
- Can formal verification integrate into the inference pipeline without killing throughput?

These are open research questions at the intersection of compiler theory, type systems, and machine learning. They won't be answered by one team.

## Series Conclusion

Across eleven articles, we traveled:

1. **Problem** — every token costs quadratically more
2. **BPE** — Python wastes 46% on syntactic overhead
3. **Constrained decoding** — correctness can be guaranteed for free
4. **Compilation** — JIT gives 2.8–5.9× speedup
5. **Types** — Hindley-Milner: zero annotations, 100% guarantees
6. **Synoema** — everything together, 937 tests, open source
7. **Future** — from text generation to structural compilation
8. **Token benchmark** — 16 algorithms, 5 languages, every token counted
9. **Runtime benchmark** — JIT vs interpreters, real execution data
10. **LLM generation** — can models learn a new language in-context?
11. **Cost calculator** — real dollar savings for engineering teams

This is not a product launch. It's a research direction that we think deserves exploration — and we're publishing everything (code, benchmarks, data, articles) so others can build on it, challenge it, or take it somewhere we haven't imagined.

If the intersection of compilers and LLMs excites you — the codebase is open, the tests are green, and the hardest problems are still unsolved.

---

*Part 7 of "Token Economics of Code" by @andbubnov. All code on GitHub under MIT License.*

---

## Footnotes

[^finetune]: **Fine-tuning** — additionally training an already-trained LLM on specialized data. Requires far less data and compute than training from scratch.

[^incremental]: **Incremental constraint checking** — verifying constraints by reusing results from previous steps instead of recalculating from scratch. Critical for million-token contexts.

[^bijection]: **Bijection** — a one-to-one correspondence between two sets. If representation A is bijective to representation B, every program in A has exactly one counterpart in B and vice versa. Lossless conversion.

[^verification]: **Formal verification** — mathematical proof that a program meets its specification. For a sort function: output is a permutation of input, and it's ordered. Used today in aviation, cryptography, and the seL4 kernel. For LLM generation — active research area.

## Glossary

| Term | Explanation |
|------|-----------|
| **Fine-tuning** | Additional LLM training on specialized data |
| **AST-level generation** | Generating structural program nodes instead of text |
| **Incremental checking** | Constraint verification reusing previous results |
| **Bijection** | One-to-one correspondence. Lossless format conversion |
| **Formal verification** | Mathematical proof of program correctness |
| **Agentic pipeline** | Chain: LLM → code → compilation → execution → result back to LLM |
