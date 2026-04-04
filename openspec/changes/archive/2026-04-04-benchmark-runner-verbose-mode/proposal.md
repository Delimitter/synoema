# Proposal: Benchmark Runner Verbose Mode

## Problem

The benchmark runner (`benchmarks/runner/`) currently outputs compact per-task progress lines (e.g., `[B] quicksort: synoema=12ms python=45ms`) and phase-level summaries. When debugging benchmark issues — failed runs, unexpected timings, missing languages — there is no way to see detailed execution info: which commands are being run, individual run timings, warm-up durations, compilation output, or script stderr.

## Solution

Add a `--verbose` (`-v`) flag to the `run` subcommand that enables detailed diagnostic output to stderr without changing the default compact behavior.

## What Changes

1. **CLI**: Add `--verbose` / `-v` boolean flag to `Command::Run`
2. **Phase propagation**: Thread `verbose: bool` through `phases::tokens::run()`, `phases::runtime::run()`, `phases::llm::run()`
3. **Verbose output** (stderr only):
   - Token phase: show python command being executed, script stderr
   - Runtime phase: show command per language, each warm-up/measured run timing, compilation commands for C++
   - LLM phase: show API request details (model, prompt length), per-attempt results
4. **Telemetry**: No changes to summary tables (they always print)

## Impact

- **Files**: `main.rs`, `phases/tokens.rs`, `phases/runtime.rs`, `phases/llm.rs`
- **No new dependencies**
- **No breaking changes** — default behavior unchanged
