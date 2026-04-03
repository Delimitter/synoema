# Constrained Decoding E2E Pipeline

End-to-end pipeline for validating and benchmarking Synoema's GBNF grammar with real LLM inference engines.

## Quick Start

```bash
# 1. Generate with llama.cpp (CPU)
./lang/tools/constrained/generate_llama.sh \
  --model path/to/model.gguf \
  --output-dir generated/ \
  --count 20

# 2. Validate generated programs
python3 lang/tools/constrained/validate_e2e.py \
  --input-dir generated/ \
  --report

# 3. Or use Docker Compose (GPU)
cd lang/tools/constrained
docker compose up -d sglang
python3 generate_sglang.py --server http://localhost:30000 --output-dir generated/
python3 validate_e2e.py --input-dir generated/ --report
```

## Components

| File | Purpose |
|------|---------|
| `prompts/prompts.json` | 100 prompts (10 tasks x 10 variations) |
| `validate_e2e.py` | Validation: parse + typecheck + run each .sno |
| `generate_llama.sh` | llama.cpp batch generation (CPU) |
| `generate_sglang.py` | SGLang generation (GPU, OpenAI API) |
| `benchmark.py` | Constrained vs unconstrained comparison |
| `docker-compose.yml` | SGLang + llama.cpp + validator containers |
| `Dockerfile.validator` | Builds Synoema + Python tooling |

## Prompt Database

10 task categories by complexity:

| # | Task | Complexity | Features |
|---|------|-----------|----------|
| 1 | Constant | Trivial | literal |
| 2 | Arithmetic | Simple | function_def |
| 3 | Factorial | Medium | recursion, pattern_match |
| 4 | List operations | Medium | lists, cons |
| 5 | String processing | Medium | strings, show, ++ |
| 6 | Records | Medium-High | records, field_access |
| 7 | ADTs | High | adt, type_def |
| 8 | Higher-order | High | lambda, pipe |
| 9 | Quicksort | High | list_comprehension |
| 10 | Module + types | Very High | module, type_sig |

## Validation Pipeline

```
Generated .sno → Parse (--errors json) → Typecheck → Run → Report (JSON + human)
```

Metrics collected:
- `syntax_rate` — fraction that parse successfully
- `type_rate` — fraction that typecheck
- `run_rate` — fraction that run without error
- `avg_parse_time_ms` — average validation time
- `errors_by_code` — distribution of error codes

## Benchmark

```bash
python3 benchmark.py --server http://localhost:30000 --output-dir bench/ --report
```

Runs the same prompts with and without grammar constraint, comparing:
- Syntax/type/run correctness rates
- Latency overhead from grammar
- Token count differences

## CI Integration

`.github/workflows/grammar-e2e.yml` triggers on changes to:
- `synoema.gbnf`
- `synoema-parser/` or `synoema-lexer/`

Steps: build -> parser tests -> validate all examples -> grammar stats.

Full E2E with llama.cpp runs on push to main (requires model download).
