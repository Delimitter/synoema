# Speculative Decoding + Constrained Grammar: The 10x Inference Speedup Nobody's Talking About

![Cover](images/cover_13.png)

## When Draft Models Know the Grammar, Acceptance Rates Skyrocket

---

> **Who this is for.** ML engineers running inference infrastructure, researchers working on speculative decoding, and anyone who wants to understand why grammar-aware speculative decoding could be the next major inference optimization.

---

Speculative decoding delivers 2-3x speedup in production. Grammar-constrained decoding guarantees correctness at near-zero cost. What happens when you combine them?

## The Speculative Decoding Primer

The basic idea: a small, fast "draft" model proposes N tokens. The large "target" model verifies all N in a single forward pass. If the draft tokens match what the target would have generated, you get N tokens for the cost of 1 forward pass.

```
STANDARD DECODE               SPECULATIVE DECODE
═══════════════                ══════════════════

Target model → tok₁            Draft model → tok₁ tok₂ tok₃ tok₄ tok₅
Target model → tok₂            Target model verifies all 5 at once
Target model → tok₃            Accept: tok₁ tok₂ tok₃ ✓
Target model → tok₄            Reject: tok₄ ✗ (resample)
Target model → tok₅
                               Result: 3 tokens for 1 target forward pass
5 forward passes               = 3x speedup (for this batch)
```

The speedup depends on the **acceptance rate** — the probability that the draft model's token matches what the target would generate.

## Grammar Constraints Change the Math

Without constraints, the draft model must predict from the full vocabulary (~100K tokens). With a grammar constraint, the valid set shrinks dramatically:

| Context | Valid next tokens | Vocabulary reduction |
|---------|------------------|---------------------|
| Free-form English | ~100,000 | — |
| Python code | ~10,000 | 10x |
| Synoema (grammar-constrained) | 50-500 | 200-2000x |

When the grammar says the next token must be one of `[let, fn, case, (, [, identifier]`, even a tiny draft model can predict it correctly. The acceptance rate approaches 1.0 for grammar-deterministic positions.

## The Compound Speedup

```
Speculative decoding alone:        2-3x speedup
Grammar-constrained decoding:      100% correctness, ~0 overhead
Combined:                          ???

Expected: acceptance rate jumps from ~60% to ~85-95%
          for grammar-predictable positions

Theoretical max speedup:           5-8x for structured code generation
```

This is not purely theoretical. TensorRT-LLM has confirmed that XGrammar (structured output) is compatible with speculative decoding. The infrastructure exists.

## Why Synoema Is Positioned for This

Three properties make Synoema uniquely suited:

**1. BPE alignment eliminates bridge tokens.** Every grammar symbol is exactly one BPE token. The draft model never needs to predict partial tokens that span grammar boundaries — eliminating a major source of rejection in constrained speculative decoding.

**2. Deterministic CFG provides predictable structure.** Synoema's grammar is a Deterministic Context-Free Grammar (DCFG). At each generation step, the grammar state machine narrows the valid token set to a small, well-defined set. This predictability is exactly what draft models need.

**3. Hindley-Milner types add a second constraint layer.** Beyond grammar, type information further constrains which identifiers and expressions are valid at each position. This reduces the effective vocabulary even more, pushing acceptance rates higher.

## Production Readiness

The components exist today:

| Component | Status | Where |
|-----------|--------|-------|
| Speculative decoding | Production | vLLM, SGLang, TensorRT-LLM |
| XGrammar (GPU constrained decoding) | Production | vLLM, SGLang, TensorRT-LLM |
| Synoema GBNF grammar | Available | `tools/constrained/synoema.gbnf` |
| Combined spec decode + grammar | Available | TensorRT-LLM (Jan 2025) |

What's missing: benchmarks of the combined approach specifically for code generation. This is an open experiment.

## The Experiment We Want to Run

```
Setup:
  Target: GPT-4o / Llama-70B
  Draft: Llama-8B / Qwen-7B
  Grammar: synoema.gbnf (via XGrammar)
  Tasks: 9 code generation tasks from our benchmark suite

Measure:
  1. Speculative decode acceptance rate (with vs without grammar)
  2. End-to-end tokens/sec (with vs without grammar)
  3. Total latency per generated program
  4. Correctness rate comparison
```

If grammar-constrained speculative decoding achieves >90% acceptance rate for Synoema generation, it would represent a fundamentally new performance tier for LLM code generation — one where the language design itself acts as a hardware accelerator.

## What's Next

Next in the series: the future of code generation — how token efficiency, type constraints, JIT compilation, and hardware acceleration converge into an agentic computation pipeline.

---

*Part 13 of "Token Economics of Code" by @andbubnov. Sources: ICLR 2026, MLSys 2025, NVIDIA.*

---

## Glossary

| Term | Explanation |
|------|-----------|
| **Speculative decoding** | Draft model proposes tokens, target model verifies in batch |
| **Acceptance rate** | Probability that draft token matches target's choice |
| **XGrammar** | GPU-accelerated grammar enforcement. Near-zero overhead |
| **Bridge tokens** | BPE tokens spanning multiple grammar symbols. Cause distribution distortion |
| **DCFG** | Deterministic Context-Free Grammar. At each state, one valid parse path |
| **Draft model** | Small, fast model (7-8B params) that proposes candidate tokens |
| **Target model** | Large, accurate model (70B+ params) that verifies candidates |
