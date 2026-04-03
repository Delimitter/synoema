---
id: tasks
type: tasks
status: done
---

# Tasks: Region Inference

## Checklist

- [x] **T1: Runtime — region stack в Arena**
  - `synoema-codegen/src/runtime.rs`
  - Добавить `region_stack: [usize; 64]`, `region_depth: usize` в `Arena`
  - Реализовать `Arena::region_enter()` / `Arena::region_exit()`
  - `pub extern "C" fn synoema_region_enter() -> i64`
  - `pub extern "C" fn synoema_region_exit() -> i64`
  - ~40 LOC

- [x] **T2: Register region FFI в compiler.rs**
  - `synoema-codegen/src/compiler.rs`
  - `builder.symbol("synoema_region_enter", ...)`
  - `builder.symbol("synoema_region_exit", ...)`
  - `declare_runtime_functions`: добавить declarations (sig0 → i64)
  - ~10 LOC

- [x] **T3: TCO auto-regions в compiler.rs**
  - `synoema-codegen/src/compiler.rs` — в compile_top_level_function
  - После создания loop_block: emit `call synoema_region_enter()`
  - Перед каждым jump к loop_block (tail-call path): emit `call synoema_region_exit()`
  - НЕ вставлять region_exit перед return path (base case) — аллокации могут быть частью return value
  - ~30 LOC

- [x] **T4: CoreExpr::Region в Core IR**
  - `synoema-core/src/core_ir.rs`
  - Добавить `Region(Box<CoreExpr>)` в enum `CoreExpr`
  - Добавить Display impl: `"(region {})"`
  - ~10 LOC

- [x] **T5: Escape analysis — `mentions()` helper**
  - `synoema-core/src/optimize.rs`
  - `fn mentions(var: &str, expr: &CoreExpr) -> bool` — проверяет, упоминается ли var свободно в expr
  - Рекурсивно обходит все варианты CoreExpr
  - Учитывает shadowing в Let/LetRec/Lam/Case
  - ~40 LOC

- [x] **T6: Escape analysis — `escapes()` function**
  - `synoema-core/src/optimize.rs`
  - `fn escapes(var: &str, body: &CoreExpr) -> bool`
  - Return position: `Var(n) == var` → escapes
  - Data structures: `MkList`, `Record`, `Con` args with mentions → escapes
  - Let chain: transitive escape check
  - Lambda: captured → escapes (conservative)
  - Known consuming builtins: `length`, `sum`, `str_len`, `show`, `print` → doesn't escape
  - ~50 LOC

- [x] **T7: allocates_heap() helper**
  - `synoema-core/src/optimize.rs`
  - `fn allocates_heap(expr: &CoreExpr) -> bool`
  - `MkList(non-empty)`, `Record`, `App(App(PrimOp(Cons|Concat|Range|Show), _), _)` → true
  - `Lit`, `Var`, `PrimOp` → false
  - ~20 LOC

- [x] **T8: Region annotation pass**
  - `synoema-core/src/optimize.rs`
  - `pub fn annotate_regions(program: CoreProgram) -> CoreProgram`
  - `fn annotate_expr(expr: CoreExpr) -> CoreExpr`
  - Для `Let(name, val, body)`: если `allocates_heap(val) && !escapes(name, body)` → `Region(Let(...))`
  - Рекурсивно обходит Lam, Case, LetRec, App
  - ~60 LOC

- [x] **T9: Codegen для CoreExpr::Region**
  - `synoema-codegen/src/compiler.rs` — в `compile_expr`
  - `CoreExpr::Region(body)`: emit `call region_enter`, compile body, emit `call region_exit`, return body result
  - ~20 LOC

- [x] **T10: Pipeline integration**
  - `synoema-codegen/src/lib.rs` — вызвать `annotate_regions()` после `optimize_program()` перед codegen
  - Только для JIT path, не для interpreter
  - ~5 LOC

- [x] **T11: Tests — TCO auto-regions**
  - `synoema-codegen/tests/stress.rs` или `synoema-codegen/src/lib.rs`
  - Тест: tail-recursive loop с 100K итераций, каждая аллоцирует list → не переполняет 8MB arena
  - Тест: `countdown n = ? n == 0 -> 0 : countdown (n - 1)` с n=1M → работает
  - Тест: `sum_acc xs acc = case xs of Nil -> acc; Cons h t -> sum_acc t (acc + h)` — per-iteration cleanup
  - ≥3 теста

- [x] **T12: Tests — escape analysis**
  - `synoema-core/src/tests.rs`
  - Тест: `let x = [1 2 3] in length x` → escapes() == false для x
  - Тест: `let x = [1 2 3] in x` → escapes() == true для x
  - Тест: `let x = [1 2 3] in head x : x` → escapes() == true
  - Тест: `let x = show 42 in str_len x` → escapes() == false
  - Тест: `let x = [1 2] in \_ -> x` → escapes() == true (closure capture)
  - ≥5 тестов

- [x] **T13: Tests — region annotation**
  - `synoema-core/src/tests.rs`
  - Тест: `let x = MkList [1 2 3] in length x` → annotated with Region
  - Тест: `let x = MkList [1 2 3] in x` → NOT annotated (escapes)
  - ≥2 теста

- [x] **T14: Tests — JIT region codegen**
  - `synoema-codegen/src/lib.rs` tests
  - Тест: функция с non-escaping let → корректный результат
  - Тест: `let xs = [1..100] in length xs` → JIT result == 100
  - Тест: `let s = show 42 in str_len s` → JIT result == 2
  - ≥3 теста

- [x] **T15: Documentation (rule 7a)**
  - `context/PROJECT_STATE.md` — обновить статус (region inference)
  - `context/PHASES.md` — добавить Phase: Region Inference
  - `context/ARCHITECTURE.md` — обновить секцию memory management
  - `CLAUDE.md` — обновить test count

- [x] **T16: Final verification**
  - `cargo test` — все тесты pass, 0 warnings
  - `cargo clippy` — 0 warnings
  - Existing benchmarks не деградировали
