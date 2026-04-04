# Benchmarks

Comparative benchmarks: Synoema vs Python, JavaScript, TypeScript, C++.

## Prerequisites

**Required for all phases:**
- Rust toolchain (`cargo build` must work)
- Python 3.9+ with `tiktoken`: `pip3 install tiktoken`

**Required for Phase B (runtime):**
- `python3`, `node`, `npx tsx`, `g++` in PATH
- Synoema release binary: `cd lang && cargo build --release -p synoema-repl`

**Required for Phase C (LLM generation) — one of:**
- OpenRouter: `pip3 install openai` + API key ([openrouter.ai](https://openrouter.ai))
- Ollama (local): [ollama.com](https://ollama.com) installed + `pip3 install openai`

## Quick Start

```bash
cd benchmarks

# Token efficiency only (fastest, no runtime deps)
cargo run --manifest-path runner/Cargo.toml -- run --phases token

# Token + runtime (no API key needed)
cargo run --manifest-path runner/Cargo.toml -- run --phases token,runtime

# Full suite including LLM code generation (OpenRouter)
cargo run --manifest-path runner/Cargo.toml -- run --all --openrouter-key YOUR_KEY

# Full suite with local ollama (no API key needed)
cargo run --manifest-path runner/Cargo.toml -- run --all --ollama
```

## CLI Reference

```
synoema-bench run [OPTIONS]

OPTIONS:
  --all                         Run all phases (token + runtime + llm)
  --phases <token,runtime,llm>  Select phases to run
  --openrouter-key <KEY>        OpenRouter API key (required for llm phase)
  --models <model1,model2>      Filter LLM models by name substring
  --tier <frontier|mid|weak>    Filter LLM models by tier
  --tasks <task1,task2>         Run only specified tasks
  --repeats <N>                 Override repeat count (default: 5)
  -v, --verbose                 Show commands, individual run timings, script stderr
  --ollama                      Use local ollama for Phase C (no API key needed)
  --ollama-model <MODEL>        Ollama model (default: qwen3:8b)
```

**Examples:**

```bash
# One task, one model, quick test
cargo run --manifest-path runner/Cargo.toml -- run --phases llm --models gpt-4o-mini --tasks factorial --openrouter-key KEY

# Only weak models
cargo run --manifest-path runner/Cargo.toml -- run --phases llm --tier weak --openrouter-key KEY

# More repeats for statistical significance
cargo run --manifest-path runner/Cargo.toml -- run --all --repeats 10 --openrouter-key KEY

# Verbose: see exact commands, warmup/run timings, script stderr
cargo run --manifest-path runner/Cargo.toml -- run --all -v --openrouter-key KEY

# Local ollama with custom model
cargo run --manifest-path runner/Cargo.toml -- run --phases llm --ollama --ollama-model qwen3:8b
```

## Phases

### A. Token Efficiency (static, parallel)

Counts BPE tokens (cl100k_base via tiktoken) for identical algorithms across 5 languages. 16 tasks.

### B. Runtime Performance (sequential)

Measures execution time (median of 5 runs, 3 warm-ups discarded). 12 tasks. Synoema JIT vs Python 3 vs Node.js vs tsx vs g++ -O2.

### C. LLM Code Generation (sequential)

Sends identical prompts to LLM models and validates generated code. Two backends:

- **OpenRouter** (default): 10 models across 3 tiers, requires API key
- **Ollama** (local): any locally-served model, no API key needed. Auto-pulls the model if not present. Uses ollama's OpenAI-compatible endpoint (`localhost:11434/v1`)

9 tasks, 5 repeats. Includes pre-flight dependency check and early exit after 10 consecutive failures.

**Models (3 tiers):**

| Tier | Models |
|------|--------|
| Frontier | GPT-4o, Gemini 2.5 Pro, Qwen3 Max |
| Mid | GPT-4o-mini, DeepSeek V3, Qwen3 Coder, Llama 4 Maverick |
| Weak | Qwen3.5 9B, LFM 1.2B (free), Reka Edge 7B |

## Tasks

| Task | A | B | C | Tests |
|------|---|---|---|-------|
| factorial | x | x | x | Recursion, base case |
| fibonacci | x | x | x | Pattern matching |
| quicksort | x | x | x | Lists, HOF, comprehensions |
| mergesort | x | x | | Divide & conquer |
| collatz | x | x | | Iteration, modulo |
| gcd | x | x | | Euclid's algorithm |
| fizzbuzz | x | x | x | Branching, strings |
| filter_map | x | x | x | Pipes, lambdas |
| binary_search | x | x | x | Index-based search |
| tree_traverse | x | x | | ADT, recursion |
| matrix_mult | | x | | Pure compute |
| string_ops | x | x | | String operations |
| json_build | x | | | Data structures |
| error_handling | x | | x | Result vs try/catch |
| pattern_match | x | | x | ADT vs class hierarchy |
| type_definition | x | | x | Custom types |

## Output

Results are saved to `benchmarks/results/<date>_run_<NNN>/`:

| File | Content |
|------|---------|
| `summary.txt` | Compact tables (same as terminal output) |
| `details.txt` | Full report: per-task breakdowns, per-model LLM results |
| `raw.json` | Machine-readable data for further analysis |

## Methodology

- **Token counting**: tiktoken cl100k_base (exact, not approximate). SPDX headers stripped for fair comparison.
- **Runtime**: subprocess timing via `std::time::Instant`. 3 warm-up runs discarded, 5 measured, median reported. C++ compiled with `-O2 -std=c++17`. Synoema uses pre-built release binary.
- **LLM generation**: OpenRouter API (OpenAI-compatible). Temperature 0.2. Synoema prompts include `docs/llm/synoema.md` as context (in-context learning). Validation: compile/parse check + run + output comparison.

## Troubleshooting

**`Error: tiktoken not installed`** — run `pip3 install tiktoken`

**`Error: openai not installed`** — run `pip3 install openai` (only needed for Phase C)

**Phase C: all 0/5 syntax, 0/5 correct** — check your OpenRouter API key is valid and has credits

**Phase C aborts after 10 failures** — pre-flight check passed but API calls fail. Check network, key validity, model availability on OpenRouter.

**`ollama is not installed or not in PATH`** — install from [ollama.com](https://ollama.com) and ensure `ollama --version` works

**Ollama model pull fails** — check disk space and that `ollama serve` is running

**Synoema runtime very slow** — ensure release binary exists: `cd lang && cargo build --release -p synoema-repl`. Without it, the runner falls back to debug build or cargo run.

**`node` / `g++` not found** — Phase B skips languages whose runtime is not in PATH. Install Node.js and Xcode Command Line Tools.

## Caveats

- TypeScript via `tsx` includes startup overhead (not representative of production TS)
- C++ compiled with `-O2` (not `-O3` or `-Ofast`)
- LLM results are stochastic — 5 repeats provide directional data, not precise measurements
- Synoema is underrepresented in LLM training data; in-context learning partially compensates
- Runtime benchmarks include JIT compilation time for Synoema
