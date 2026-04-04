# Proposal: Benchmark Documentation Update

## Problem
The benchmark suite was added (`benchmarks/`) but documentation has gaps:
1. `docs/benchmarks.md` — missing prerequisites (pip install tiktoken openai), missing CLI reference
2. `CLAUDE.md` — navigation table doesn't list `docs/benchmarks.md` or `benchmarks/`
3. `context/PROJECT_STATE.md` — repo structure outdated (no `benchmarks/` at root), metrics don't reference new suite
4. `docs/benchmarks.md` — no mention of early exit on failures, no troubleshooting section
5. `CONTRIBUTING.md` — missing prerequisites for running benchmarks

## Scope
Documentation-only changes. No code modifications.
- Update `docs/benchmarks.md` — add prerequisites, CLI reference, troubleshooting
- Update `CLAUDE.md` — add benchmarks to navigation table
- Update `context/PROJECT_STATE.md` — add benchmarks/ to repo structure, reference new benchmark suite
- Update `CONTRIBUTING.md` — add prerequisites note
