# Tasks: Ollama Test Integration

## Detection & Core Logic
- [x] Add `ollama_available()` function to `benchmarks/runner/src/phases/llm.rs`
- [x] Add `ensure_model(model, verbose)` function to `benchmarks/runner/src/phases/llm.rs`
- [x] Add `--ollama` and `--ollama-model` CLI flags to `benchmarks/runner/src/main.rs`
- [x] Wire ollama path in main.rs run dispatch (use localhost:11434 as base_url)

## Script Changes
- [x] Add `--base-url` argument to `benchmarks/scripts/llm_generate.py`

## Tests
- [x] Add `test_ollama_detection` test (always runs, checks function returns bool)
- [x] Add `test_ensure_model` test (`#[ignore]`, pulls qwen3:8b if ollama present)
- [x] Add `test_ollama_single_task` test (`#[ignore]`, full round-trip)

## Documentation
- [x] Update `docs/benchmarks.md` — add Ollama section to prerequisites and CLI reference
- [x] Update `docs/testing.md` — mention ollama-gated tests
