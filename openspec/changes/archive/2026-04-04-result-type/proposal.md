# Proposal: result-type — Prelude mechanism + Result ADT + error builtin

## Problem Statement

Synoema не имеет стандартного механизма обработки ошибок. LLM, генерирующая код, вынуждена использовать pattern matching на ad-hoc ADT или паниковать при ошибках. Нет prelude — стандартная библиотека типов и функций не загружается автоматически.

Без Result type и prelude:
- Каждый проект переопределяет `Maybe`, `Result` заново
- Нет `error` для аварийного завершения с сообщением
- LLM не может предполагать наличие стандартных комбинаторов

## Scope

### 1. Prelude mechanism
- `lang/prelude/prelude.sno` — встроен в бинарник через `include_str!`
- Загружается перед пользовательской программой в interpreter
- Prelude-определения доступны без import
- Не ломает существующие 864+ тестов

### 2. Result type (в prelude)
- `Result a e = Ok a | Err e` — стандартный ADT
- Комбинаторы: `map_ok`, `map_err`, `unwrap`, `unwrap_or`, `is_ok`, `is_err`, `and_then`
- Pipe-friendly: `Ok 5 |> map_ok (\n -> n * 2) |> unwrap_or 0`

### 3. `error` builtin
- Interpreter: `error : String -> a` — panic с сообщением
- JIT: `synoema_error(msg_ptr: i64) -> !` — FFI runtime function
- Используется в `unwrap` для паники при `Err`

## What is NOT in scope
- Map type (separate change, depends on Result)
- JSON parsing (depends on Result + Map)
- JIT compilation of prelude (interpreter-first; JIT already handles ADTs from user code)
- Prelude as importable module (transparent, not a module)

## Success Criteria
1. `map_ok (\x -> x + 1) (Ok 5) == Ok 6`
2. `unwrap_or 0 (Err "fail") == 0`
3. `and_then (\x -> Ok (x * 2)) (Ok 5) == Ok 10`
4. `Ok 5 |> map_ok (\n -> n * 2) |> unwrap_or 0` == 10
5. `error "boom"` → runtime panic с "boom"
6. `cargo test` — 0 failures, 0 warnings
7. Все существующие тесты зелёные
