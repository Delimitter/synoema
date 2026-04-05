# Tasks: 20 Tool Driver Examples

## Batch 1: Data Structures (5)
- [x] Create `lang/examples/string_utils.sno` — string reverse, pad, repeat, contains
- [x] Create `lang/examples/list_toolkit.sno` — chunk, flatten, unique, partition, zip_with
- [x] Create `lang/examples/tuple_ops.sno` — Pair ops: zip, unzip, assoc list, lookup
- [x] Create `lang/examples/stack.sno` — Stack ADT: push, pop, peek, is_empty
- [x] Create `lang/examples/queue.sno` — Queue via two-stack trick

## Batch 2: Algorithms (5)
- [x] Create `lang/examples/math_toolkit.sno` — primes sieve, gcd, lcm, fib sequence
- [x] Create `lang/examples/sorting.sno` — mergesort + insertion sort
- [x] Create `lang/examples/searching.sno` — binary search on sorted list
- [x] Create `lang/examples/tree.sno` — BST: insert, member, inorder traversal
- [x] Create `lang/examples/graph.sno` — adjacency list, DFS reachability

## Batch 3: Patterns (5)
- [x] Create `lang/examples/pipeline.sno` — multi-stage |> data pipeline
- [x] Create `lang/examples/validator.sno` — composable Result-chain validation
- [x] Create `lang/examples/error_handling.sno` — and_then, recovery, sequence_results
- [x] Create `lang/examples/state_machine.sno` — traffic light FSM with ADT
- [x] Create `lang/examples/cache.sno` — memoization table via Map

## Batch 4: Tools (5)
- [x] Create `lang/examples/config.sno` — env-based config with defaults
- [x] Create `lang/examples/json_transform.sno` — JSON parse/transform pipeline
- [x] Create `lang/examples/cli_tool.sno` — CLI args processing with dispatch
- [x] Create `lang/examples/parser.sno` — recursive-descent tokenizer + evaluator
- [x] Create `lang/examples/serializer.sno` — record to string serialization

## Validation
- [x] Run all 20 examples via `cargo run -p synoema-repl -- run` — 0 errors
- [x] Run `cargo test` — 871 pass, 0 failures, 18 ignored (preexisting)

## Bugs Discovered During Automation
1. **3-equation pattern matching bug**: Functions with 3+ equations using mixed literal/variable patterns return wrong values (e.g., `merge [] ys = ys; merge xs [] = xs; merge (x:xs) (y:ys) = ...` — `merge [3] []` returns `[]` instead of `[3]`). Workaround: use ternary `? length xs == 0 -> ys : ...`.
2. **`index` argument order**: Docs say `[a] -> Int -> a` but actual signature is `Int -> [a] -> a`. Correct usage: `index 2 xs` not `index xs 2`.
3. **`json_encode` missing**: Listed in docs but not implemented as builtin.
4. **`not` missing**: Listed as builtin but not available. Workaround: `x == false`.
5. **`++` type ambiguity in recursion**: Recursive string functions using `++` fail type check because `++` is overloaded for List and String. Workaround: use string interpolation `"${a}${b}"`.
6. **Nested list patterns in constructors**: `(MkStack [])` is not valid. Workaround: `? length xs == 0`.
7. **`:` ambiguity in ternary**: Cons `:` inside ternary branches must be parenthesized: `? cond -> (x : xs) : (y : ys)`.
8. **`[f x]` parses as 2-element list**: `[show state]` is parsed as `[show, state]`. Fix: `[(show state)]`.
