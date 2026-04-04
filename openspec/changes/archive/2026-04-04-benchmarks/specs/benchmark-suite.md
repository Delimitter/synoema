# Spec: Benchmark Suite

## Languages
Synoema, Python, JavaScript, TypeScript, C++

## Phases

### Phase A: Token Efficiency (static analysis, parallel)
- Tokenizer: cl100k_base via tiktoken (Python)
- 16 tasks: identical algorithms across all languages
- Metrics: token count, line count, char count, vs-Synoema %

### Phase B: Runtime Performance (sequential, isolated)
- 12 tasks (subset of A, excluding token-only tasks)
- Protocol: 3 warm-up runs discarded, 5 measured runs
- Metrics: median, p5, p95, peak RSS (KB), vs-Synoema ratio
- Execution: Synoema JIT, Python 3, Node.js, ts-node, g++ -O2

### Phase C: LLM Code Generation (sequential, API rate limits)
- 9 tasks (3 easy, 3 medium, 3 hard)
- 10 models via OpenRouter (3 tiers)
- 5 repeats per (task × language × model)
- Metrics: syntax validity %, semantic correctness %, avg tokens, avg cost ($), retries
- Validation: language-specific compile/run + output comparison

## Models (OpenRouter)

### Tier 1 — Frontier
- openai/gpt-4o
- google/gemini-2.5-pro
- qwen/qwen3-max-thinking

### Tier 2 — Mid
- openai/gpt-4o-mini
- deepseek/deepseek-chat-v3-0324
- qwen/qwen3-coder-next
- meta-llama/llama-4-maverick

### Tier 3 — Weak
- qwen/qwen3.5-9b
- liquid/lfm-2.5-1.2b-instruct:free
- rekaai/reka-edge

## Benchmark Tasks

| # | Task | Phase A | Phase B | Phase C | Purpose |
|---|------|---------|---------|---------|---------|
| 1 | factorial | x | x | x | recursion, base case |
| 2 | fibonacci | x | x | x | recursion, TCO |
| 3 | quicksort | x | x | x | lists, pattern match, HOF |
| 4 | mergesort | x | x | | divide & conquer |
| 5 | collatz | x | x | | loops/recursion, numbers |
| 6 | gcd | x | x | | math, iteration |
| 7 | fizzbuzz | x | x | x | branching, strings |
| 8 | filter+map | x | x | x | HOF, lambdas |
| 9 | binary_search | x | x | x | arrays, comparison |
| 10 | tree_traverse | x | x | | ADT, recursion |
| 11 | matrix_mult | | x | | pure compute |
| 12 | string_ops | x | x | | concat, search, split |
| 13 | json_build | x | | | data structures |
| 14 | error_handling | x | | x | Result/try-catch |
| 15 | pattern_match | x | | x | match vs if-chains |
| 16 | type_definition | x | | x | ADT vs class/struct |

## Output Files

```
benchmarks/results/<YYYY-MM-DD>_run_<NNN>/
├── summary.txt      — compact terminal-friendly table
├── details.txt      — full human-readable report (sections A/B/C/D)
└── raw.json         — machine-readable data
```

## API Key
Command-line argument only: `--openrouter-key KEY`
Phases A and B work without a key. Phase C requires it or is skipped with warning.
