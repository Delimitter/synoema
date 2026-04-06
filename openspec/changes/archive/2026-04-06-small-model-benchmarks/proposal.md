---
id: proposal
type: proposal
status: done
---

# Small Model Benchmarks

## Why

Current benchmark Phase C tests 10 models via OpenRouter API but doesn't isolate the small model experience:
1. No GBNF on/off comparison (API doesn't support grammar constraints)
2. No comparison of reference sizes (1800 tok vs compact 800 tok)
3. Doesn't test the error feedback multi-pass loop
4. Ollama support exists but only for a single model at a time

Goal: add Phase D to the benchmark runner — local small model testing via ollama with GBNF support, multi-reference comparison, and multi-pass error correction.

## What Changes

- New phase `size` in benchmark runner (Phase D: Model Size Reduction)
- Ollama-based local inference with multiple small models
- 3 configurations per model: baseline (1800 tok) / compact (800 tok) / compact + multi-pass
- Results table: model × config × syntax_rate × type_rate × run_rate
- New CLI flag: `--phases size` and `--size-models <model1,model2>`

## Capabilities

### New Capabilities

- `phase-d-benchmark`: Phase D benchmark runner — local ollama inference, multi-model, multi-config comparison with syntax/type/run rates
- `multi-pass-benchmark`: Benchmark configuration that tests error feedback loop (generate → check → retry with llm_hint)

### Modified Capabilities

- `benchmark-runner`: Add Phase D support (--phases token,runtime,llm,size)

## Scope

### In Scope

- Phase D runner code in `benchmarks/runner/src/phases/size.rs`
- CLI integration (--phases size, --size-models)
- Report generation for Phase D results
- Default models: qwen2.5-coder:0.5b, qwen2.5-coder:1.5b, qwen2.5-coder:3b, qwen2.5-coder:7b
- 3 configs: baseline / compact-ref / compact-ref + multi-pass (2 retries)
- Use existing 9 LLM-eligible tasks from Phase C

### Out of Scope

- GBNF constrained decoding (ollama doesn't natively support GBNF — needs llama.cpp direct integration, future phase)
- Fine-tuned models (testing zero-shot/few-shot only)
- New tasks beyond existing 9
- Cross-language comparison (covered by archived small-model-proof change)
