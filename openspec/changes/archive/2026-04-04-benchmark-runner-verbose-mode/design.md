# Design: Benchmark Runner Verbose Mode

## Approach

Thread a `verbose: bool` parameter from CLI through all phase functions. In verbose mode, emit extra `eprintln!` lines prefixed with `    ` (4-space indent) to visually distinguish them from standard progress.

## Decisions

1. **Flag**: `--verbose` / `-v` on the `Run` subcommand (not global). Simple boolean, no log levels.
2. **Propagation**: Pass `verbose` as a parameter to each phase `run()` function. No global state, no env var.
3. **Output format**: All verbose output goes to stderr via `eprintln!`, indented with 4 spaces for visual hierarchy under the `[A]`/`[B]`/`[C]` task lines.
4. **What to show**:
   - Token phase: full python command, script stderr if non-empty
   - Runtime phase: exact command being executed, each individual run timing (warm-up + measured), C++ compile command
   - LLM phase: model name, prompt token count, per-attempt response tokens and correctness

## Non-Goals

- Log levels (debug, trace) — overkill for a benchmark runner
- File-based logging — results already go to files
- Colored output — no color dependencies
