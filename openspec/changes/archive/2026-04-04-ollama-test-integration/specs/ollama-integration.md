# Spec: Ollama Integration

## Detection
- Check: `ollama --version` exits with code 0
- Model check: `ollama list` output contains model name
- Auto-pull: `ollama pull qwen3:8b` if model not present (with user-visible progress)

## CLI Extension
- New flag: `--ollama` — use local ollama instead of OpenRouter
- New flag: `--ollama-model <MODEL>` — override model (default: `qwen3:8b`)
- `--ollama` and `--openrouter-key` are mutually exclusive for Phase C

## Integration with Phase C
- Ollama exposes OpenAI-compatible API at `http://localhost:11434/v1`
- llm_generate.py already uses openai SDK — reuse with base_url override
- Model name format for ollama: just `qwen3:8b` (no provider prefix)

## Test Coverage
- Unit test: `detect_ollama()` returns bool
- Unit test: `ensure_model()` pulls if missing
- Integration test: if ollama available, run one task with one model (factorial/synoema)
- Tests that require ollama are gated with `#[ignore]` (run with `--ignored`)
