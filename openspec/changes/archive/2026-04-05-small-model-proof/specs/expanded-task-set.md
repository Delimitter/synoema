# Expanded Task Set

## Purpose

Expand from 9 LLM-eligible tasks to 30, covering algorithmic, data structure, functional, and practical categories. All tasks have reference implementations in Synoema, Python, and Haskell.

## Requirements

### R1: Task Categories

**Existing (16 tasks, need Haskell implementations):**
1. factorial, fibonacci, quicksort, mergesort, collatz, gcd
2. fizzbuzz, filter_map, binary_search
3. tree_traverse, matrix_mult, string_ops, json_build
4. error_handling, pattern_match, type_definition

**New (14 tasks):**

Level 1 — Basics:
- `power`: integer exponentiation via recursion
- `palindrome`: check if list/string is palindrome
- `flatten`: flatten nested list structure

Level 2 — Data Structures:
- `bst_insert`: binary search tree insert + in-order traversal
- `stack_calc`: simple stack-based calculator (RPN)

Level 3 — Functional:
- `compose_chain`: compose 3+ functions, apply to list
- `group_by`: group list elements by predicate
- `scan_left`: running fold (scanl equivalent)

Level 4 — Type System:
- `maybe_chain`: chain Maybe/Option operations (map, flatMap, default)
- `either_validate`: validate with Either/Result, accumulate errors
- `record_transform`: transform/update nested records

Level 5 — Practical:
- `csv_parse`: parse simple CSV string into list of records
- `word_freq`: word frequency count from string
- `state_machine`: simple state machine (traffic light / vending machine)

### R2: Task Structure

Each task directory: `benchmarks/tasks/<name>/`
- `prompt.txt` — language-agnostic task description
- `expected_output.txt` — expected stdout
- `<name>.sno` — Synoema reference
- `<name>.py` — Python reference
- `<name>.hs` — Haskell reference

### R3: Haskell Implementations for Existing Tasks

Add `.hs` reference implementations for all 16 existing tasks that lack them.

### R4: Consistency

- All implementations produce identical output
- No external dependencies (stdlib only)
- Synoema implementations verified by `cargo test` / compiler
- Python verified by `python3`
- Haskell verified by `runghc`
