# Proposal: Benchmark Improvements Post-Analysis

## Context

Full benchmark run completed (2026-04-05_run_002): 10 models × 9 tasks × 5 languages × 5 repeats.
Results revealed systemic issues in benchmark infrastructure and LLM language reference.

## Problems Found

### P0: Critical benchmark bugs
1. **error_handling, pattern_match** — 0% correctness across ALL 5 languages (not just synoema). Broken prompts or expected_output validation.
2. **14 synoema-only tasks** missing JS/TS/C++ implementations — skew averages for token/runtime comparison.
3. **fibonacci** — 0% syntax for synoema LLM generation across all 10 models.

### P1: LLM generability gap (synoema 34% syntax vs Python 92%)
4. `docs/llm/synoema.md` lacks examples for failing tasks (binary_search, fibonacci, type_definition, filter_map).
5. **filter_map** — 58% syntax but 0% correctness. LLM generates valid code that computes wrong result.
6. Clear pattern: simple tasks (factorial 78%, fizzbuzz 78%, quicksort 64%) vs hard tasks (0%).

### P2: Infrastructure
7. **Save generated code** to disk for post-mortem analysis.
8. **--exclude-models** flag for skipping broken models.

## Scope

- Fix 2 broken benchmark tasks
- Expand LLM language reference with pattern examples
- Exclude synoema-only tasks from LLM phase (they only have synoema + python)
- Add code saving + model exclusion to runner
- Do NOT change the language itself or compiler
