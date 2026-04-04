# Tasks: Benchmark Suite

## Infrastructure
- [x] Create `benchmarks/` directory structure (runner/, scripts/, tasks/, results/)
- [x] Create `benchmarks/runner/Cargo.toml` with clap for CLI
- [x] Create `benchmarks/runner/src/main.rs` — CLI arg parsing, phase dispatch
- [x] Create `benchmarks/runner/src/telemetry.rs` — live terminal progress output
- [x] Create `benchmarks/runner/src/report.rs` — generate summary.txt, details.txt, raw.json
- [x] Create `benchmarks/runner/src/phases/mod.rs`

## Phase A: Token Counting
- [x] Create `benchmarks/scripts/token_count.py` — tiktoken cl100k_base counter
- [x] Create `benchmarks/runner/src/phases/tokens.rs` — invoke token_count.py, collect JSON

## Phase B: Runtime Performance
- [x] Create `benchmarks/runner/src/phases/runtime.rs` — subprocess timing, memory measurement

## Phase C: LLM Generation
- [x] Create `benchmarks/scripts/llm_generate.py` — OpenRouter API client + validation
- [x] Create `benchmarks/runner/src/phases/llm.rs` — orchestrate model × task × language × repeats

## Benchmark Tasks (5 languages each)
- [x] Create tasks/factorial/ (sno, py, js, ts, cpp + expected_output.txt)
- [x] Create tasks/fibonacci/
- [x] Create tasks/quicksort/
- [x] Create tasks/mergesort/
- [x] Create tasks/collatz/
- [x] Create tasks/gcd/
- [x] Create tasks/fizzbuzz/
- [x] Create tasks/filter_map/
- [x] Create tasks/binary_search/
- [x] Create tasks/tree_traverse/
- [x] Create tasks/matrix_mult/
- [x] Create tasks/string_ops/
- [x] Create tasks/json_build/
- [x] Create tasks/error_handling/
- [x] Create tasks/pattern_match/
- [x] Create tasks/type_definition/

## Configuration
- [x] Add `benchmarks/results/` to `.gitignore`

## Documentation
- [x] Create `docs/benchmarks.md` — results, methodology, how to run, caveats
- [x] Update `README.md` — link to docs/benchmarks.md in Documentation table + Why section
- [x] Update `CONTRIBUTING.md` — add "Running Benchmarks" section

## Verification
- [x] `cargo build` in benchmarks/runner/ succeeds
- [x] `cargo run -- run --phases token,runtime` works without API key
- [x] Phase C skips gracefully without --openrouter-key
- [ ] All expected_output.txt files match actual output for each language
