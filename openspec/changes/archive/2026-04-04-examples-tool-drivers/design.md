# Design: 20 Tool Driver Examples

## Architecture

All 20 files go into `lang/examples/`. No subdirectories — flat structure matches existing convention.

## Conventions

Each file follows this template:
```sno
-- SPDX-License-Identifier: MIT-0
-- Short description

--- Doc comment explaining the pattern.
--- example: main == ()

<helper definitions>

main = <expression>
```

- `main` must print output (so `run` shows results) and return `()`
- Helper functions defined before `main`
- Prefer `|>` pipe style for data transforms
- Use prelude types (Result, Pair, Map) where appropriate
- No external imports

## Validation Strategy

Each example tested in two ways:
1. `cargo run -p synoema-repl -- run <file>` — must succeed with exit 0
2. MCP `eval`/`run` tool — same code should work

## Complexity Budget

- Simple patterns (string_utils, tuple_ops): 15-30 lines
- Medium patterns (tree, validator, pipeline): 30-60 lines
- Complex patterns (parser, graph, state_machine): 50-80 lines

## Known Constraints

- No mutation → use functional patterns (fold, recursion)
- No real file I/O in examples → use env_or, print for demo
- Map type is sorted assoc list → O(n) operations, fine for examples
- String operations limited to str_len, str_slice, str_find, str_trim, str_starts_with
- No `str_split` builtin → implement via recursion on str_find
