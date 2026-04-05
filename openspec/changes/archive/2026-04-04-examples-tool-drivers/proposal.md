# Proposal: 20 Tool Driver Examples

## Problem
The `lang/examples/` directory has only 24 files, mostly demonstrating basic language features (factorial, fizzbuzz, records). There are no examples showing how Synoema handles **everyday programming patterns** — the kind of code developers write daily: data validation, configuration, pipelines, state machines, parsers, serialization, etc.

LLM users hitting the MCP `synoema://examples` resource see toy demos, not practical tool drivers.

## Scope
Create 20 new `.sno` files in `lang/examples/`, each an isolated context demonstrating one commonly-used programming pattern ("tool driver"):

1. **string_utils** — string manipulation (reverse, pad, split, join)
2. **list_toolkit** — advanced list ops (chunk, flatten, unique, zip_with, partition)
3. **math_toolkit** — primes, gcd, lcm, fibonacci sequence
4. **sorting** — mergesort + insertion sort (quicksort exists already)
5. **searching** — binary search on sorted list
6. **stack** — stack ADT via list (push, pop, peek)
7. **queue** — queue via two-stack trick
8. **tree** — binary search tree (insert, member, traversal)
9. **graph** — adjacency list + DFS
10. **json_transform** — JSON parse → transform → encode pipeline
11. **config** — env-based configuration with defaults and validation
12. **validator** — composable validators returning Result chains
13. **pipeline** — multi-stage data pipeline with |> composition
14. **state_machine** — traffic light FSM via ADT + transitions
15. **parser** — simple recursive-descent expression parser
16. **cli_tool** — CLI argument processing with help/version
17. **error_handling** — Result patterns: and_then chains, recovery, sequence
18. **cache** — memoization table using Map
19. **tuple_ops** — Pair-based operations (zip, unzip, assoc list manipulation)
20. **serializer** — record ↔ string serialization

Each file:
- Self-contained (no imports beyond prelude)
- Has `main =` entry point
- Has doc comments (`---`) explaining the pattern
- Has at least 1 doctest (`--- example:`)
- Demonstrates a real-world use case, not just syntax

## Out of Scope
- Modifying existing examples
- Changing MCP server code (auto-discovery already works)
- Performance benchmarks for examples
- Multi-file examples (import system already covered by `imports/`)

## Validation
- Each file must pass `cargo run -p synoema-repl -- run <file>`
- Each file must appear in MCP `synoema://examples` index
- Total: ≥20 new examples, 0 runtime errors
