# Proposal: Comparative Benchmark Suite

## Problem
Synoema claims 46% token savings vs Python and 4.4x JIT speedup, but:
- Existing benchmarks are ad-hoc (Python script with manual token counter, 3-row performance table)
- No comparison with JS, TS, C++
- No LLM code generation quality tests
- Not runnable from a single command
- No structured output

## Scope
- Benchmark runner: Rust CLI + Python scripts for token counting and LLM API
- 16 benchmark tasks implemented in 5 languages (Synoema, Python, JS, TS, C++)
- 3 test phases: A (token efficiency), B (runtime performance), C (LLM code generation)
- 10 LLM models via OpenRouter API (3 frontier, 4 mid, 3 weak)
- Single command launch: `cargo run -- run --all --openrouter-key KEY`
- Output: summary.txt (terminal table) + details.txt (full report) + raw.json
- Documentation: docs/benchmarks.md + README.md + CONTRIBUTING.md updates

## Out of Scope
- GBNF constrained decoding benchmarks (separate future work)
- Haskell comparison (removed — not in target set)
- C#/Java (too much infrastructure overhead for marginal insight)
- Automated CI integration (manual runs only for now)
