---
id: tasks
type: tasks
status: done
---

# Tasks: Constrained Decoding E2E Pipeline

## Checklist

- [x] **T1: Prompt database**
  - Created `lang/tools/constrained/prompts/prompts.json` with 10 tasks × 10 variations = 100 prompts
  - Tasks: constant, arithmetic, factorial, list_ops, strings, records, ADTs, higher-order, quicksort, modules
  - JSON format with task name, complexity, expected_features, prompt text

- [x] **T2: Validation script**
  - `lang/tools/constrained/validate_e2e.py`
  - Pipeline: parse (cargo run --errors json) → typecheck → collect metrics
  - Output: JSON report + human summary with syntax/type/run rates

- [x] **T3: llama.cpp integration**
  - `lang/tools/constrained/generate_llama.sh`
  - Batch generation with GBNF grammar constraint
  - Parameters: model path, output dir, count, temperature, max tokens

- [x] **T4: SGLang integration**
  - `lang/tools/constrained/generate_sglang.py`
  - OpenAI-compatible API client with `extra_body={"ebnf": grammar}`
  - Supports constrained and unconstrained modes

- [x] **T5: Docker-compose setup**
  - `lang/tools/constrained/docker-compose.yml`
  - Services: sglang (GPU), llama (CPU), validator, benchmark
  - `lang/tools/constrained/Dockerfile.validator`

- [x] **T6: Benchmark runner**
  - `lang/tools/constrained/benchmark.py`
  - Constrained vs unconstrained comparison
  - Metrics: latency, tokens, syntax/type/run rates, overhead %

- [x] **T7: CI workflow**
  - `.github/workflows/grammar-e2e.yml`
  - Trigger: changes to GBNF, parser, or lexer
  - Jobs: grammar-validation (always) + e2e-llama (main only)

- [x] **T8: Documentation**
  - `docs/constrained-decoding-e2e.md` — setup guide, components, benchmark usage

- [x] **T9: Final verification**
  - All 771+ tests pass, 0 warnings
  - All 13 example programs validate successfully
