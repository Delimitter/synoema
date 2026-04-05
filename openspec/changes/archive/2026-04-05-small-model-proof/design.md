# Design: Small Model Proof

## Architecture Decision: Separate Script, Not Rust Runner Extension

Phase D uses a **standalone Python script** (`benchmarks/scripts/size_benchmark.py`) instead of extending the Rust runner. Reasons:

1. llama.cpp interaction is simpler from Python (HTTP API to `llama-server`)
2. GBNF grammar loading is a llama-server feature, not an application-level concern
3. Analysis and charting need Python (matplotlib/pandas)
4. Keeps the Rust runner focused on token/runtime/API-based LLM phases
5. The Rust runner can invoke the Python script as a phase if desired later

## Inference Architecture

```
┌──────────────┐     HTTP      ┌──────────────────┐
│ size_bench-  │ ───────────── │ llama-server     │
│ mark.py      │  /v1/chat/    │ --model X.gguf   │
│              │  completions  │ --grammar Y.gbnf  │
│              │               │ --port 8090       │
└──────────────┘               └──────────────────┘
       │
       ▼
┌──────────────┐     subprocess    ┌──────────────┐
│ Validate:    │ ─────────────── │ synoema run   │
│ syntax +     │                  │ python3       │
│ typecheck +  │                  │ runghc        │
│ correctness  │                  └──────────────┘
└──────────────┘
       │
       ▼
  results.json
```

### llama-server Management

The script does NOT manage llama-server lifecycle. User starts it manually:

```bash
llama-server -m models/qwen2.5-coder-3b-instruct-q4_k_m.gguf \
  --port 8090 --ctx-size 4096
```

For GBNF mode, the grammar is sent per-request via the API `grammar` parameter (llama.cpp supports this in the chat completions endpoint).

### Why Not Ollama?

Ollama doesn't support GBNF grammar constraints. llama-server is the only local inference server with native GBNF support. This is critical — constrained decoding is the core differentiator.

## Validation Pipeline

```
Generated code
    │
    ├─ Synoema: cargo run -p synoema-repl -- run <file>
    │           (checks parse + typecheck + execution)
    │
    ├─ Python:  python3 -c "compile(open('f').read(), 'f', 'exec')"
    │           then: python3 <file>
    │
    └─ Haskell: runghc <file>
            (checks parse + typecheck + execution in one step)

Compare stdout to expected_output.txt → correct: true/false
```

Synoema has a unique advantage: `--errors json` gives structured error output. We log error categories (syntax vs type vs runtime) for analysis.

## Reference Doc Strategy

Each language gets ~1800-token reference doc with identical structure:

| Section | Content |
|---------|---------|
| Overrides | Syntax that differs from "default" expectations |
| Axioms | Core principles (5-7 bullets) |
| Functions | How to define and call |
| Data types | ADTs, records, basic types |
| Pattern matching | Syntax and examples |
| List operations | Comprehensions, map/filter/fold |
| Error handling | Result/Maybe/Exception style |
| Stdlib | Key functions available |

For Python this is almost unnecessary (models know Python), but including it ensures **fair comparison** — all languages get equal context window budget.

## Task Design

### 14 New Tasks

Each new task:
- `prompt.txt`: 2-4 sentences, language-agnostic
- `expected_output.txt`: deterministic output
- Reference implementations in .sno, .py, .hs
- Verified: all three produce identical stdout

### Few-Shot Examples

For `few-shot` mode, each language has 3 canonical examples included in the prompt:
- Example 1: basic function + pattern matching (factorial)
- Example 2: list processing (filter + map)
- Example 3: ADT + error handling

These are drawn from existing benchmark tasks, not from the eval set.

## Analysis Design

### Primary Chart: Correctness vs Model Size (per language)

X-axis: model size (0.5B, 1.5B, 3B, 7B) — log scale
Y-axis: correct_rate (0-100%)
Lines: Synoema (zero-shot), Synoema (few-shot), Synoema (GBNF), Python, Haskell

### Key Metric: Size Reduction Factor

```
min_model(Python, 70%) / min_model(Synoema+GBNF, 70%) = N×
```

"Synoema reduces model size requirements by N×"

### Secondary Analysis

- Decomposition: how much does GBNF contribute vs fewer tokens vs simpler syntax?
  - GBNF contribution = constrained_rate - few_shot_rate (same model, Synoema)
  - Token contribution = estimated from token count difference
  - Syntax contribution = residual
- Per-task breakdown: which task categories benefit most?
- Error taxonomy: syntax errors vs type errors vs logic errors per language

## File Structure

```
benchmarks/
  scripts/
    size_benchmark.py          # Main Phase D script
    size_analysis.py           # Analysis + charts
  tasks/
    power/                     # 14 new task dirs
    palindrome/
    flatten/
    bst_insert/
    stack_calc/
    compose_chain/
    group_by/
    scan_left/
    maybe_chain/
    either_validate/
    record_transform/
    csv_parse/
    word_freq/
    state_machine/
  results/
    size_proof/                # Phase D results
docs/
  llm/
    python.md                  # Python ICL reference
    haskell.md                 # Haskell ICL reference
  articles/
    15_en_small_model_proof.md # Workshop paper
```

## Model Selection: Qwen2.5-Coder

Why Qwen2.5-Coder family:
1. Available in 0.5B, 1.5B, 3B, 7B — perfect size spectrum
2. Code-specialized (trained on code corpus)
3. Instruction-tuned variants available
4. GGUF quantizations on HuggingFace
5. Strong baselines on HumanEval/MBPP

Alternative if Qwen unavailable: DeepSeek-Coder-V2 (1.3B, 6.7B) or Phi-3 (3.8B).

## Statistical Rigor

- 5 attempts per (model, language, mode, task)
- Report mean + std for all rates
- Wilson score confidence intervals for proportions
- Total N per cell = 150 (30 tasks × 5 attempts) — sufficient for meaningful comparisons
- Effect size (Cohen's h) for cross-language differences
