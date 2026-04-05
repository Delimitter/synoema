# Tasks: Benchmark Improvements Post-Analysis

## P0: Fix benchmark tasks

- [x] 1. Fix `error_handling` prompt — added explicit output format to prompt.txt
- [x] 2. Fix `pattern_match` prompt — added explicit values, dimensions, and output format
- [x] 3. Exclude synoema-only tasks from LLM phase — already excluded (LLM_TASKS only lists tasks with 5/5 impls)

## P1: Improve LLM language reference

- [x] 4. Added "Complete Examples" section to docs/llm/synoema.md — 7 solved patterns (fibonacci, binary_search, shapes/ADT, linked list, filter_map, error handling, quicksort)
- [x] 5. Expanded "Gotchas" section — added show, integer division, index, multi-line output (10 items now)
- [x] 6. ADT + pattern matching example included in section 17

## P2: Benchmark infrastructure

- [x] 7. Save generated code — llm_generate.py --save-dir + --attempt; runner passes results/generated/
- [x] 8. Added --exclude-models flag to CLI + resolve_models
