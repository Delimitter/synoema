# Tasks: Small Model Proof

## Phase D Benchmark Script

- [x] T1: Create `benchmarks/scripts/size_benchmark.py` — main Phase D runner
  - llama-server HTTP client (OpenAI-compatible /v1/chat/completions)
  - GBNF grammar parameter support per-request
  - Model/language/mode matrix iteration
  - Validation pipeline (syntax + correctness) for Synoema, Python, Haskell
  - JSON output per generation: {syntax_ok, type_ok, correct, tokens_out, time_ms, error, code}
  - CLI: --base-url, --tasks-dir, --language, --mode, --repeats, --output-dir, --grammar
  - Progress display

- [x] T2: Create `benchmarks/scripts/size_analysis.py` — analysis + charts
  - Read Phase D results JSON
  - Compute aggregated rates: syntax_rate, type_rate, correct_rate per (model, language, mode)
  - Generate comparison table (terminal + markdown)
  - Compute "minimum model for 70% correctness" metric

## Cross-Language Reference Docs

- [x] T3: Create `docs/llm/python.md` — Python ICL reference (~1800 tokens)
  - Same structure as synoema.md
  - Cover: functions, pattern matching (match/case), list comprehensions, error handling, stdlib

- [x] T4: Create `docs/llm/haskell.md` — Haskell ICL reference (~1800 tokens)
  - Same structure as synoema.md
  - Cover: equations, guards, pattern matching, ADTs, Maybe/Either, where, list comprehensions

## Expanded Task Set — Haskell Implementations for Existing 16 Tasks

- [x] T5: Add `.hs` implementations for existing 16 benchmark tasks
  - factorial.hs, fibonacci.hs, quicksort.hs, mergesort.hs, collatz.hs, gcd.hs
  - fizzbuzz.hs, filter_map.hs, binary_search.hs
  - tree_traverse.hs, matrix_mult.hs, string_ops.hs, json_build.hs
  - error_handling.hs, pattern_match.hs, type_definition.hs

## Expanded Task Set — 14 New Tasks (all 3 languages)

- [x] T6: Create task `power` — integer exponentiation via recursion
- [x] T7: Create task `palindrome` — check if list/string is palindrome
- [x] T8: Create task `flatten` — flatten nested list structure
- [x] T9: Create task `bst_insert` — BST insert + in-order traversal
- [x] T10: Create task `stack_calc` — RPN stack calculator
- [x] T11: Create task `compose_chain` — compose functions, apply to list
- [x] T12: Create task `group_by` — group list elements by predicate
- [x] T13: Create task `scan_left` — running fold (scanl)
- [x] T14: Create task `maybe_chain` — chain Maybe/Option operations
- [x] T15: Create task `either_validate` — validate with Either/Result
- [x] T16: Create task `record_transform` — transform/update records
- [x] T17: Create task `csv_parse` — parse CSV string
- [x] T18: Create task `word_freq` — word frequency count
- [x] T19: Create task `state_machine` — simple state machine (traffic light)

## Article

- [x] T20: Write `docs/articles/15_en_small_model_proof.md` — workshop paper draft
  - Structure: abstract, intro, background, setup, results (placeholders), analysis, related work, conclusion
  - Use [RESULT: metric] placeholders for actual numbers

## Integration

- [x] T21: Add Phase D instructions to `docs/benchmarks.md`
  - Document how to run size_benchmark.py
  - Document llama-server setup and model download

- [ ] T22: Verify all new .sno files compile and run correctly
  - Run `cargo run -p synoema-repl -- run` on all 14 new .sno files ✓ (14/14 pass)
  - Run `python3` on all 14 new .py files ✓ (14/14 pass)
  - Run `runghc` on all 14 new .hs files (pending ghc availability check)
  - Verify existing tests still pass
