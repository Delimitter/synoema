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
TextMate grammar для .sno (`vscode-extension/`).

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

## TCO в JIT
Self-recursive tail calls → jump to loop header. `TcoContext` threads through `compile_expr`/`compile_case`, propagated in tail positions (Case branches, Let/LetRec body). O(1) stack для tail-recursive функций. countdown(10M) и sum_to(1M) в JIT без stack overflow. 5 тестов.

## String Stdlib в JIT
`str_slice`, `str_find`, `str_starts_with`, `str_trim`, `str_len`, `json_escape` — все 6 функций портированы из interpreter в JIT через FFI-паттерн. Arena-allocated строки, 13 тестов.

## Phase 19 — Doc-as-Code
`---` doc-comments сохраняются в AST (Token::DocComment → Decl.doc: Vec<String>). Doctests (`--- example: expr == val`) проверяются при `synoema test`. `synoema doc` генерирует Markdown из кода. Guide metadata (`--- guide:`, `--- order:`). 0 влияния на runtime/JIT. 12 новых тестов (6 lexer + 6 parser).

## Phase 20 — LLM Cost Reduction v1
Пять фич для удешевления LLM-использования:
1. **Stdlib catalog:** `docs/llm/stdlib.md` — машиночитаемый каталог всех builtins с типами для weak LLMs.
2. **Type aliases:** `type Pos = {x: Int, y: Int}` — `KwType` в lexer, `Decl::TypeAlias` в AST, expansion в type checker через `resolve_type_expr`. Parametric aliases, recursive alias detection. 8 тестов.
3. **Error recovery:** `parse_recovering()` — parser собирает все ошибки за один проход (skip-to-next-decl). `infer_program_recovering()` — type checker продолжает после ошибки. 6 тестов.
4. **String interpolation:** `"hello ${name}"` — lexer: `StringFragment`/`InterpStart`/`InterpEnd` tokens, brace depth tracking. Parser: `ExprKind::StringInterp`. Desugar: `show` + `++` chain. Escape: `\$`. 8 тестов.
5. **Multi-file imports:** `import "path.sno"` — `ImportDecl` в AST, `resolve_imports` рекурсивный resolver, cycle detection (visited set), diamond caching (по canonical path). 4 теста.

## Phase 21 — Region Inference
Автоматическое управление памятью в JIT через region inference. Два уровня:
1. **TCO auto-regions:** tail-recursive loops автоматически освобождают per-iteration heap через `region_enter`/`region_exit` FFI. Вставка в codegen при компиляции TCO-функций.
2. **Escape analysis + region annotation:** Core IR pass определяет non-escaping `let`-bindings (через `escapes()` + `allocates_heap()`), оборачивает их в `CoreExpr::Region`. JIT emits `region_enter`/`region_exit` вокруг scope.
Runtime: region stack в Arena (массив saved offsets, depth до 64). 15 новых тестов.

## Phase 22 — Built-in Testing
Встроенная система тестирования: три уровня от конкретного к абстрактному.
1. **Doctests** (существовали) — `--- example: expr == val` в doc-комментариях.
2. **Test declarations** — `test "name" = <Bool-expr>` — standalone тесты как top-level декларации. `Decl::Test` в AST.
3. **Property-based testing** — `prop vars -> body` — type-driven генерация 100 случайных входов. `ExprKind::Prop` + `ExprKind::Implies`. LCG генератор (без crate rand).
Keywords: `test`, `prop`, `when` — все 1 BPE-токен. Runner: `synoema test <path> [--filter <str>]`. Поддержка `implies` для conditional properties.

## Phase 23 — Bug fixes + new builtins
1. **Multi-arg numeric pattern matching:** `f 0 xs = ...` теперь правильно матчит только когда первый аргумент = 0 (раньше матчил все вызовы).
2. **Singleton list pattern `[x]`:** parser поддерживает `[x]`, `[x y z]` и т.д. — десахаризация в nested Cons.
3. **Where-bindings в else-ветке ternary:** `? cond -> expr : name = val ...` теперь работает.
4. **JIT list display:** `[1 2 3]` отображается корректно (раньше показывал raw tagged pointers).
5. **Новые builtins (eval/run/jit):** `zip`, `index`, `take`, `drop`, `reverse`. `foldl` ранее был только в eval — теперь также в run/jit.
6. **Benchmark fixes:** 5 .sno файлов переписаны на поддерживаемый синтаксис, `-std=c++17` для C++, явное FAIL-логирование.
937 тестов (было ~875–890).

