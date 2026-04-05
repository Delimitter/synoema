# Delta Spec: Examples Catalog

## Added: 20 Tool Driver Examples

Each file in `lang/examples/` follows the convention:
- Header: `-- SPDX-License-Identifier: MIT-0`
- Doc comment block: `--- Description of the pattern`
- At least 1 doctest: `--- example: main == ()`
- Self-contained, prelude only
- `main = <expr>` entry point

### File List

| # | File | Pattern | Key Features |
|---|------|---------|-------------|
| 1 | string_utils.sno | String manipulation | str_len, str_slice, ++, recursion |
| 2 | list_toolkit.sno | List processing | chunk, flatten, unique, partition |
| 3 | math_toolkit.sno | Math utilities | primes, gcd, lcm, fib |
| 4 | sorting.sno | Sort algorithms | mergesort, insertion sort |
| 5 | searching.sno | Search algorithms | binary search |
| 6 | stack.sno | Stack ADT | push, pop, peek via list |
| 7 | queue.sno | Queue ADT | two-stack queue |
| 8 | tree.sno | Binary search tree | insert, member, inorder |
| 9 | graph.sno | Graph algorithms | adjacency list, DFS |
| 10 | json_transform.sno | JSON pipeline | parse → transform → encode |
| 11 | config.sno | Configuration | env_or, defaults, validation |
| 12 | validator.sno | Data validation | Result chains, composable |
| 13 | pipeline.sno | Data pipeline | |> composition |
| 14 | state_machine.sno | FSM | ADT states + transitions |
| 15 | parser.sno | Recursive descent | expression parser |
| 16 | cli_tool.sno | CLI processing | args, help, dispatch |
| 17 | error_handling.sno | Error patterns | and_then, recovery, sequence |
| 18 | cache.sno | Memoization | Map-based lookup table |
| 19 | tuple_ops.sno | Pair operations | zip, unzip, assoc list |
| 20 | serializer.sno | Serialization | record → string → record |

### MCP Integration
No code changes needed. `synoema://examples` auto-discovers all `.sno` files in `lang/examples/`.
