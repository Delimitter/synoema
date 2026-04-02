# Plan: Synoema Defect Correction (Pre-Phase 12)

> Аудит кодовой базы, апрель 2026. Исправить до начала Phase 12 (type classes).

## Контекст

Проект на этапе Phase 11.5 → Phase 12. 373 теста, 0 failures, 0 warnings.
Аудит выявил 6 верифицированных дефектов — от дырки в системе типов до дрифта документации.
Дефекты 1-3 затрагивают фундамент типовой системы и должны быть исправлены ДО Phase 12.

---

## Дефекты (по убыванию приоритета)

### 1. CRITICAL — Row polymorphism: пропуск унификации при r1 == r2
**Файл:** `lang/crates/synoema-types/src/unify.rs:128-137`

**Баг:** Когда два открытых record-типа имеют одну и ту же row-переменную, но разные exclusive-поля, типы этих полей НЕ унифицируются. Позволяет `{x: Int, age: Int | r}` пройти унификацию с `{x: Int, age: String | r}`.

**Фикс:** В ветке `r1 == r2` — найти поля, которые есть в обоих `extra_in_1` и `extra_in_2`, и унифицировать их типы. Поля, уникальные для одной стороны — допустимы (расширяют ряд). Поля в обоих наборах — должны иметь совместимые типы.

**Тест:** `{x: Int, y: Bool | r}` НЕ унифицируется с `{x: Int, y: String | r}` при одной row-переменной.

---

### 2. HIGH — Non-exhaustive patterns молча возвращают 0
**Файл:** `lang/crates/synoema-core/src/desugar.rs:161`

**Баг:** `build_equation_chain` возвращает `CoreExpr::Lit(Lit::Int(0))` как fallback. Функция `f True = 1` для `f False` молча вернёт 0 вместо runtime error.

**Фикс:** Добавить `CoreExpr::RuntimeError(String)` или вызов `synoema_match_error` FFI.

**Затронутые файлы:**
- `core/src/core_ir.rs` — новый вариант `RuntimeError(String)` в `CoreExpr`
- `core/src/desugar.rs:161` — emit `RuntimeError` вместо `Lit(0)`
- `eval/src/eval.rs` — обработка `RuntimeError` (panic с сообщением)
- `codegen/src/compiler.rs` — обработка `RuntimeError` (вызов FFI trap)
- `codegen/src/runtime.rs` — добавить `synoema_match_error` extern fn

**Тест:** Non-exhaustive match → runtime error (и interpreter, и JIT).

---

### 3. MEDIUM — Constructor patterns не проверяются по типу
**Файл:** `lang/crates/synoema-types/src/infer.rs:539-554`

**Баг:** `infer_pattern` для `Pat::Con` создаёт fresh type variable вместо lookup-а типа конструктора из env. Позволяет `Just(x, y)` пройти type check при `Just : a -> Maybe a`.

**Фикс:** Передать type environment в `infer_pattern`. Для `Pat::Con(name, sub_pats)`:
1. Lookup `name` в env
2. Instantiate scheme → function type
3. Unify типы sub-pattern'ов с аргументами конструктора
4. Вернуть result type конструктора

**Тест:** `Just(x, y)` отклоняется type checker'ом когда `Just : a -> Maybe a`.

---

### 4. MEDIUM — Arena fallback аллокации утекают
**Файл:** `lang/crates/synoema-codegen/src/runtime.rs:48-52`

**Баг:** При переполнении арены — fallback в системный `alloc()`. Но `arena_reset()` не освобождает fallback-аллокации.

**Фикс:** Добавить `Vec<(*mut u8, Layout)>` в Arena для трекинга fallback'ов. В `reset()` — `dealloc()` каждого перед очисткой вектора.

---

### 5. LOW — Record field hash collision (теоретический)
**Файл:** `lang/crates/synoema-codegen/src/runtime.rs:611-621`

**Баг:** `synoema_record_get` ищет поле только по FNV-1a хешу. Коллизия → неверное поле.

**Оценка:** Вероятность коллизии FNV-1a 64-bit для коротких имён ~10^-19. Теоретический риск. Добавить compile-time проверку уникальности хешей в одном record.

---

### 6. LOW — Документация рассинхронизирована
- `context/PROJECT_STATE.md:75` — "368 тестов" → 373
- `context/PROJECT_STATE.md:218` — "264 теста" → 373
- `context/DEVELOPMENT_GUIDE.md:10,200` — "264 теста" → 373
- `context/PROJECT_STATE.md` — "12 примеров" → 13
- `context/PROJECT_STATE.md` — Cranelift "0.113" → 0.104

---

### 7. LOW — Clippy warnings (4 штуки)
- `token.rs:2` — empty line after doc comment
- `types.rs:257` — `Default` impl для `TyVarGen`
- `infer.rs:23` — `Default` impl для `Infer`
- `infer.rs:237` — `&self` only used in recursion

---

## Порядок выполнения

| # | Дефект | Приоритет | Блокирует Phase 12? |
|---|--------|-----------|---------------------|
| 1 | Row polymorphism soundness | CRITICAL | Да |
| 2 | Non-exhaustive patterns | HIGH | Да |
| 3 | Constructor pattern types | MEDIUM | Да |
| 4 | Arena fallback leak | MEDIUM | Нет |
| 5 | Clippy warnings | LOW | Нет |
| 6 | Documentation drift | LOW | Нет |
| 7 | Hash collision check | LOW | Нет |

## Верификация

```bash
cd lang && cargo test          # 373+ тестов зелёные
cd lang && cargo clippy         # 0 warnings
cd lang && cargo build          # 0 warnings
```

Новые тесты: ~6-8 (type checker, interpreter, codegen).

## False positives (отклонённые находки)

- Record pattern matching в interpreter — корректно для open records
- List comprehension guard — правильная семантика через concatMap
- Tagged pointer scheme — стандартная практика, 8-byte alignment гарантирует свободные биты
- `transmute` для JIT function pointer — стандартный паттерн Cranelift
