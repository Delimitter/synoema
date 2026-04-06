---
id: design
type: design
status: done
---

# Design: Small Model Benchmarks

## Architecture

Phase D reuses existing LLM phase infrastructure (ollama API, task discovery, validation) but adds:
1. Multi-model iteration (4 models by default)
2. Multi-config per model (3 configurations)
3. Multi-pass error correction loop (config 3)
4. Extended metrics (type_rate, run_rate in addition to syntax_rate)

## Pipeline

```
For each model in [0.5b, 1.5b, 3b, 7b]:
  For each config in [baseline, compact, compact+multipass]:
    For each task in [9 LLM tasks]:
      For each repeat in [1..5]:
        1. Build prompt (reference + task prompt)
        2. Call ollama /api/generate
        3. Save .sno to temp file
        4. Validate: parse → typecheck → run
        5. If config=multipass AND parse/type failed:
           a. Get JSON error (--errors json)
           b. Build retry prompt with error + llm_hint
           c. Retry up to 2 times (temperature: 0.7 → 0.4 → 0.2)
        6. Record: syntax_ok, type_ok, run_ok, tokens, latency
```

## Reference Configs

| Config | Reference Doc | Tokens | Multi-pass |
|--------|--------------|--------|------------|
| `baseline` | `docs/llm/synoema.md` | ~1800 | No |
| `compact` | `docs/llm/synoema-compact.md` | ~800 | No |
| `multipass` | `docs/llm/synoema-compact.md` | ~800 | Yes (2 retries) |

## Multi-Pass Error Correction

For `multipass` config, on parse or typecheck failure:

```
retry_prompt = f"""Your code has an error:
{error.llm_hint}
{error.did_you_mean or ""}

Fix the code:
{previous_code}"""
```

Temperature decay: 0.7 → 0.4 → 0.2 across attempts.

## Module Structure

New file: `benchmarks/runner/src/phases/size.rs`

```rust
pub struct SizeConfig {
    name: String,          // "baseline" | "compact" | "multipass"
    reference_path: PathBuf,
    multi_pass: bool,
    max_retries: u32,      // 0 for single-pass, 2 for multipass
}

pub struct SizeResults {
    models: Vec<SizeModelResults>,
}

pub struct SizeModelResults {
    model: String,
    configs: Vec<SizeConfigResults>,
}

pub struct SizeConfigResults {
    config: String,
    tasks: Vec<SizeTaskResult>,
    avg_syntax_pct: f64,
    avg_type_pct: f64,
    avg_run_pct: f64,
}
```

## CLI Integration

```
--phases size               # Run Phase D only
--phases token,size         # Combine phases
--size-models qwen2.5-coder:3b,qwen2.5-coder:7b  # Subset of models
```

Default models if `--size-models` not specified:
`qwen2.5-coder:0.5b,qwen2.5-coder:1.5b,qwen2.5-coder:3b,qwen2.5-coder:7b`

## Validation

Reuse existing validation from Phase C:
1. Parse: `synoema --errors json run <file>` — exit code 0 = syntax ok
2. Typecheck: implicit in `run` — type errors in JSON output
3. Run: compare output vs `expected_output.txt`

Extended: capture error JSON for multi-pass retry prompt construction.

## Report

Phase D report section added to markdown output:

```
## D. Model Size Reduction

| Model | Config | Syntax % | Type % | Run % | Avg Tokens |
|-------|--------|----------|--------|-------|------------|
| qwen2.5-coder:0.5b | baseline | ... | ... | ... | ... |
| qwen2.5-coder:0.5b | compact | ... | ... | ... | ... |
| qwen2.5-coder:0.5b | multipass | ... | ... | ... | ... |
| ... | ... | ... | ... | ... | ... |
```
