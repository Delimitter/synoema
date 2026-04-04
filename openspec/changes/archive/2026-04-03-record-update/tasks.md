# Tasks: Record Update Syntax

## Task 1: Лексер — Token::DotDotDot

- [ ] В `synoema-lexer/src/token.rs`: добавить `DotDotDot` в enum `Token`
- [ ] В `synoema-lexer/src/scanner.rs`: в блоке `b'.'` добавить проверку `...` перед `..`
- [ ] Добавить тест в `synoema-lexer/src/tests.rs`: `"..."` tokenizes to `[DotDotDot]`
- [ ] BPE verify: `...` = 1 токен на cl100k_base

## Task 2: AST — ExprKind::RecordUpdate

- [ ] В `synoema-parser/src/ast.rs`: добавить в `ExprKind`:
  ```rust
  RecordUpdate {
      base: Box<Expr>,
      updates: Vec<(String, Expr)>,
  }
  ```
- [ ] В `synoema-parser/src/parser.rs`: в блоке `Token::LBrace` — если следующий `Token::DotDotDot`, парсить как RecordUpdate
- [ ] Синтаксис: `{ ... expr , field = val ( , field = val )* }`
- [ ] Тест парсера: `{...r, x=1}` → `RecordUpdate { base: Var("r"), updates: [("x", 1)] }`
- [ ] Тест парсера: `{...{x=1,y=2}, x=10, y=20}` — вложенный base

## Task 3: Типизация — RecordUpdate

- [ ] В `synoema-types/src/infer.rs`: добавить case для `ExprKind::RecordUpdate`
- [ ] Стратегия: infer base → unify с open record из updates → return base type
- [ ] Тест: `{...{x=1, y=2}, x=10}` → type `{x: Int, y: Int}` ✓
- [ ] Тест: `{...{x=1}, z=5}` → type error (z не существует в base) ✓
- [ ] Тест: `{...{x=1}, x="hello"}` → type error (wrong type) ✓

## Task 4: Eval — interpreter

- [ ] В `synoema-eval/src/eval.rs`: добавить case `ExprKind::RecordUpdate { base, updates }`:
  - eval base → `Value::Record(mut fields)`
  - для каждого `(name, expr)` в updates: eval expr, найти поле по имени, заменить значение
  - если поле не найдено → `Err("field 'X' not found in record")`
  - вернуть `Value::Record(fields)`
- [ ] Тест: `{...{x=1, y=2}, x=10} == {x=10, y=2}`
- [ ] Тест: `{...{x=1, y=2, z=3}, x=10, y=20} == {x=10, y=20, z=3}`
- [ ] Тест: `let r = {a=1, b=2} in {...r, a=99} == {a=99, b=2}`

## Task 5: Desugar — CoreExpr::RecordUpdate

- [ ] В `synoema-core/src/core_ir.rs`: добавить в `CoreExpr`:
  ```rust
  RecordUpdate {
      base: Box<CoreExpr>,
      updates: Vec<(String, CoreExpr)>, // (field_name, value_expr)
  }
  ```
- [ ] В `synoema-core/src/desugar.rs`: добавить case `ExprKind::RecordUpdate`:
  ```rust
  ExprKind::RecordUpdate { base, updates } =>
      CoreExpr::RecordUpdate {
          base: Box::new(desugar_expr(fresh, base)),
          updates: updates.iter().map(|(n, e)| (n.clone(), desugar_expr(fresh, e))).collect(),
      }
  ```

## Task 6: JIT — runtime + compiler

- [ ] В `synoema-codegen/src/runtime.rs`: добавить `synoema_record_clone(rec: i64) -> i64`
  - Считать `len` из rec
  - Аллоцировать новый record `synoema_record_new(len)`
  - Скопировать все `(hash, val)` пары из rec в новый
- [ ] В `synoema-codegen/src/runtime.rs`: добавить `synoema_record_set_field(rec: i64, hash: i64, val: i64)`
  - Линейный поиск по hash, перезаписать val (in-place, только на свежем record)
  - Паника если hash не найден
- [ ] В `synoema-codegen/src/compiler.rs`: зарегистрировать `synoema_record_clone` + `synoema_record_set_field` как symbols + объявить функции
- [ ] В `synoema-codegen/src/compiler.rs`: добавить case `CoreExpr::RecordUpdate { base, updates }`:
  - Compile base → base_ptr
  - Вызвать `synoema_record_clone(base_ptr)` → new_ptr
  - Для каждого `(field_name, val_expr)` в updates:
    - Compile val_expr → val
    - hash = `field_name_hash(field_name)` (compile-time константа)
    - Вызвать `synoema_record_set_field(new_ptr, hash, val)`
  - Вернуть new_ptr
- [ ] Тест JIT: `{...{x=1, y=2}, x=10} == {x=10, y=2}` (через `jit eval`)
- [ ] Тест JIT: `{...{x=1, y=2, z=3}, x=10, y=20} == {x=10, y=20, z=3}`

## Task 7: GBNF + документация

- [ ] Обновить `tools/constrained/synoema.gbnf`: добавить record-update rule
- [ ] Обновить `docs/llm/synoema.md`: добавить `{...r, x = val}` в секцию Records
- [ ] Обновить `context/PROJECT_STATE.md`: отметить record update как implemented

## Task 8: Cargo test

- [ ] Запустить `cargo test` из `lang/` — 0 failures, 0 warnings
- [ ] Все существующие тесты зелёные
