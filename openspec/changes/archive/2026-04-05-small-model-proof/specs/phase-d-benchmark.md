# Phase D: Model Size Reduction Benchmark

## Purpose

Prove that Synoema's language design reduces the minimum model size required for correct code generation, compared to Python and Haskell.

## Requirements

### R1: Local Inference via llama.cpp

- Use llama.cpp server (`llama-server`) for inference
- Support GGUF model files (Q4_K_M quantization)
- GBNF grammar loading for Synoema constrained decoding
- Configurable: model path, context size, temperature, grammar file

### R2: Model Matrix

4 model sizes from Qwen2.5-Coder family:
- 0.5B (qwen2.5-coder-0.5b-instruct)
- 1.5B (qwen2.5-coder-1.5b-instruct)
- 3B (qwen2.5-coder-3b-instruct)
- 7B (qwen2.5-coder-7b-instruct)

All Q4_K_M quantization for consistent comparison.

### R3: Language Matrix

3 target languages:
- Synoema (with and without GBNF)
- Python (no GBNF — context-sensitive grammar, not expressible as CFG)
- Haskell (no GBNF — layout rule is context-sensitive)

### R4: Mode Matrix

3 generation modes:
- `zero-shot`: system prompt + task description + language reference doc only
- `few-shot`: system prompt + task description + language reference doc + 3 example programs
- `constrained`: same as few-shot + GBNF grammar enforcement (Synoema only)

### R5: Task Set

30 tasks total (existing 16 + 14 new). Each task has:
- `prompt.txt` — task description
- `expected_output.txt` — expected stdout
- Reference implementations: `task.sno`, `task.py`, `task.hs`

### R6: Metrics

Per (model, language, mode, task, attempt):
- `syntax_ok` (bool): parses without error
- `type_ok` (bool): passes type checking (Synoema: typecheck, Python: mypy optional, Haskell: ghc)
- `correct` (bool): output matches expected
- `tokens_out` (int): generated tokens
- `time_ms` (float): generation time

Aggregated:
- `syntax_rate` = syntax_ok / total
- `type_rate` = type_ok / total
- `correct_rate` = correct / total
- `min_model_for_X` = smallest model achieving X% correctness

### R7: Execution

- 5 attempts per (model, language, mode, task) combination
- Temperature: 0.3 (consistent across all)
- Max tokens: 512
- Timeout: 60 seconds per generation
- Total: 4 models x 3 languages x 3 modes x 30 tasks x 5 attempts = 5,400 generations
- (Synoema has 3 modes, Python/Haskell have 2 modes → actual: 4 x (1x3 + 2x2) x 30 x 5 = 4,200)

### R8: Output

- Raw JSON: per-generation results
- Summary tables: syntax/type/correct rates per (model, language, mode)
- Cross-language comparison: same model, different languages
- Key metric: "minimum model size for 70% correctness" per language
