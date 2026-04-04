# Proposal: Ollama Test Integration

## Problem
Benchmark suite Phase C (LLM generation) requires OpenRouter API key and internet access. There is no way to test LLM code generation locally. Ollama is a popular local LLM runner — if available on the developer's machine, it should be usable for local benchmark runs with qwen3:8b.

## Scope
- Add ollama availability detection to benchmark runner (preflight check)
- Add `--ollama` flag to benchmark CLI for local LLM testing
- Auto-pull `qwen3:8b` if ollama is present but model missing
- Add integration test that verifies ollama detection logic
- Update documentation: docs/benchmarks.md, docs/testing.md

## Out of Scope
- Replacing OpenRouter integration (stays as-is)
- Adding other local LLM runners (vllm, llama.cpp directly)
- GPU configuration or performance tuning
