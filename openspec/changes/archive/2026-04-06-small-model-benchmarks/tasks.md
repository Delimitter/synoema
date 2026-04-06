---
id: tasks
type: tasks
status: done
---

# Tasks: Small Model Benchmarks

- [x] **T1: Create `benchmarks/runner/src/phases/size.rs`**
  - SizeConfig, SizeResults, SizeModelResults, SizeConfigResults, SizeTaskResult structs
  - `run()` with multi-model, multi-config loops
  - Ollama API integration via curl
  - Validation: parse → typecheck → run via synoema binary
  - Code extraction from LLM responses (```sno blocks)

- [x] **T2: Multi-pass error correction**
  - On failure: extract error hint from stderr
  - Build retry prompt with error feedback + previous code
  - Temperature decay: 0.7 → 0.4 → 0.2
  - Max 2 retries for multipass config

- [x] **T3: CLI integration**
  - `size` added to phases enum (--phases size)
  - `--size-models` CLI flag added
  - Phase D dispatch wired in main.rs
  - Registered in mod.rs

- [x] **T4: Report generation**
  - Phase D section in report.rs summary builder
  - Table: model × config × syntax% × type% × run% × avg_tokens × task_count
  - AllResults extended with `size: Option<SizeResults>`

- [x] **T5: Verify compilation**
  - `cargo build` clean (0 warnings)
  - `cargo test` all 1011 tests pass (0 failures)
