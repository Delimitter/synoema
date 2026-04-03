# Завершённые фазы

## Phase 9.2 — Closures в JIT
Lambda lifting, indirect calls, map/filter.

## Phase 9.3 — Строки в JIT
Tagged ptr bit 1, StrNode, show/++/length, fizzbuzz.

## Phase 9.4 — Records
Interpreter + JIT: RecordNode heap, FNV-hash field access.

## Phase 9.5 — Модули
`mod Name` + `use Name (...)` — lexical namespacing, desugaring pass.

## Phase 10.1 — TCO в interpreter
Iterative eval loop + 64MB stack thread.

## Phase 10.2 — Constant folding/DCE
Core IR optimizer: `2+3→5`, `? true -> x : y → x`.

## Phase 10.3 — Arena allocator
Region-based, no malloc leaks, arena_reset после каждого запуска.

## String ==
`synoema_val_eq` runtime dispatch — int и string.

## Phase 11.1 — ADTs в JIT
ConNode heap alloc, tag comparison, field extraction, 6 tests.

## Phase 11.2 — Row polymorphism
Rémy-style row unification для records, 7 type tests.

## Phase 11.3 — Nested ADT patterns в JIT
Nested constructor matching, 2 codegen tests.

## Phase 11.4 — Full ADT pattern matching в JIT
Literal sub-patterns, triple nesting, recursive `bind_sub_pat`.

## Phase 11.5 — String literal patterns в JIT
Top-level + sub-patterns inside constructors, 5 tests.

## Phase 12a — Float в JIT
FloatNode heap-alloc, tag=0x04, 10 tests: arithmetic + comparisons + cond.

## Phase 12b — Record patterns в JIT
CorePat::Record в compile_case + bind_sub_pat, 5 tests.

## Phase 13 — Float Eq/Ord/Show
Interpreter + 19 tests (float ==, <, >, show, ADT+float).

## `**` operator
Power + float math builtins (sqrt, floor, ceil, abs, round) — interpreter + JIT, 28 tests.

## VS Code extension
TextMate grammar для .sno (`tools/vscode-extension/`).

## Phase 14a — IO/Effects в interpreter
`()` unit, `print` (∀a. a → ()), `;` sequence op, `readline`, 11 tests.

## Phase 14b — IO в JIT
`synoema_print_val` (FFI), `synoema_readline`, `Lit::Unit → iconst(0)`, 8 tests.

## Phase 15a — JIT completeness
`show` для всех типов, `list ==` (recursive), `[a..b]` ranges, 13 tests.

## Phase 15b — show Bool/List в JIT
Compile-time fold `show true/false`, `synoema_show_list`, 8 tests.

## Phase 15c — show ADTs/Records в JIT
CON_TAG=1/RECORD_TAG=5 pointer tagging, `synoema_show_con`, 8 tests.

## Phase 16 — Type class Show dispatch в JIT
User-defined `impl Show Color` в JIT, `synoema_show_bool`, `is_bool_expr` heuristic, 10 tests.

## Phase 17 — Higher-order stdlib в JIT
`synoema_map`/`synoema_filter`/`synoema_foldl` via closure ABI, curried foldl, 8 tests.

## Phase 18 — Сетевые примитивы + строковый stdlib
Интерпретатор: `str_slice`, `str_find`, `str_starts_with`, `str_trim`, `str_len`, `json_escape`, `file_read`, `tcp_listen`, `tcp_accept`, `fd_readline`, `fd_write`, `fd_close`, `fd_popen`.
Пример: `examples/stress_server.sno` — HTTP-сервер для stress dashboard.
Type checker: линейные типы (`LinearArrow`, `LinearDuplicate`, `LinearUnused`).
Диагностика: `synoema-diagnostic` crate, JSON/human рендереры, span в type errors.

