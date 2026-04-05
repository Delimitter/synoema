# Small Model Proof

## Why

Synoema's core thesis: a language designed for LLM code generation should reduce model size requirements. We have indirect evidence (token benchmarks, frontier LLM generation results), but no direct proof that **small models perform better with Synoema than with Python/Haskell**.

Current LLM benchmark (Phase C) tests 10 models but:
1. Uses API-only inference (OpenRouter) — no constrained decoding (GBNF)
2. Doesn't isolate the **language design** variable — models know Python from pre-training
3. Doesn't test truly small models (0.5B–3B) where the difference should be most visible
4. Only compares Synoema generation, not cross-language on the same tasks

The strongest proof: a 3B model + Synoema + GBNF achieves correctness comparable to a much larger model + Python, **without any fine-tuning**. This proves the language design itself reduces model requirements.

## What Changes

- New benchmark phase (Phase D: Model Size Reduction) using llama.cpp locally
- 4 model sizes: 0.5B, 1.5B, 3B, 7B (Qwen2.5-Coder family, GGUF)
- 3 languages: Synoema, Python, Haskell
- 3 modes: zero-shot ICL, few-shot ICL, ICL + GBNF (Synoema only)
- 30 tasks (expanded from current 9 LLM-eligible), 5 attempts each
- Python & Haskell reference docs (~1800 tokens each) for fair ICL comparison
- Analysis script producing tables and charts
- Article 15 (workshop paper format): "Does Language Design Reduce LLM Size Requirements?"

## Capabilities

### New Capabilities

- `phase-d-benchmark`: Phase D benchmark runner — llama.cpp-based local inference with GBNF support, multi-model, multi-language comparison
- `expanded-task-set`: 30 benchmark tasks with reference implementations in Synoema, Python, Haskell
- `cross-language-ref-docs`: ICL reference documents for Python and Haskell (~1800 tokens each), matching Synoema's format
- `analysis-pipeline`: Analysis script for tables, charts, statistical comparisons
- `article-15`: Workshop paper "Does Language Design Reduce LLM Size Requirements?"

### Modified Capabilities

- `benchmark-runner`: Add Phase D support to existing runner CLI (--phases token,runtime,llm,size)

## Scope

### In Scope

- Benchmark infrastructure for local llama.cpp inference with GBNF
- 14 new tasks (expand 16 existing to 30, with Python + Haskell implementations)
- Reference docs for Python and Haskell (for fair ICL comparison)
- Analysis and visualization scripts
- Article 15 draft in workshop paper format

### Out of Scope

- Fine-tuning models (this is zero-shot / few-shot only — that's the point)
- Changes to the Synoema compiler or language
- Changes to the GBNF grammar (already exists)
- Hosting or deployment of models
- Formal peer review submission (draft only)
