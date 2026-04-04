# Tasks: result-type

## T1. Prelude file

- [x] Создать `lang/prelude/prelude.sno` с Result ADT и комбинаторами
- [x] Содержимое: `Result a e = Ok a | Err e`
- [x] Комбинаторы: `map_ok`, `map_err`, `unwrap`, `unwrap_or`, `is_ok`, `is_err`, `and_then`

## T2. `error` builtin (interpreter)

- [x] В `eval.rs`: добавить `"error"` в `builtin_env()` с arity 1
- [x] В `call_builtin`: `"error"` → `Err(EvalError { kind: Type, message: msg })`
- [x] Тест: `error "boom"` → runtime error содержит "boom"

## T3. Prelude loading (interpreter)

- [x] В `lib.rs`: `const PRELUDE` через `include_str!`
- [x] В `eval_main_inner`: prepend prelude к user source перед parse
- [x] В `eval_expr`: prepend prelude аналогично (для REPL)
- [x] Тест: `is_ok (Ok 1) == true` — prelude доступна без import
- [x] Тест: все существующие тесты не ломаются

## T4. `error` builtin (JIT)

- [x] В `runtime.rs`: `pub extern "C" fn synoema_error(msg_ptr: i64) -> i64`
- [x] В `compiler.rs`: зарегистрировать `synoema_error` в `declare_runtime_functions()`
- [x] В `compiler.rs`: обработать вызов `error` как FFI call (registered as "error" → sig1)
- [x] Тест: JIT `error` — compiles and registers (panic test removed: can't unwind through JIT frames)

## T5. Prelude loading (JIT)

- [x] В `lib.rs` codegen: prepend prelude к source перед compile
- [x] Тест: JIT `unwrap (Ok 42) == 42`

## T6. Result combinator tests

- [x] `map_ok (\x -> x + 1) (Ok 5) == Ok 6`
- [x] `map_ok (\x -> x + 1) (Err "e") == Err "e"`
- [x] `map_err (\e -> e ++ "!") (Err "fail") == Err "fail!"`
- [x] `unwrap (Ok 42) == 42`
- [x] `unwrap_or 0 (Err "fail") == 0`
- [x] `unwrap_or 0 (Ok 5) == 5`
- [x] `is_ok (Ok 1) == true`
- [x] `is_ok (Err "x") == false`
- [x] `is_err (Err "x") == true`
- [x] `is_err (Ok 1) == false`
- [x] `and_then (\x -> Ok (x * 2)) (Ok 5) == Ok 10`
- [x] `and_then (\x -> Ok (x * 2)) (Err "e") == Err "e"`
- [x] Pipe: `Ok 5 |> map_ok (\n -> n * 2) |> unwrap_or 0` == 10

## T7. Documentation

- [x] Обновить `CLAUDE.md` — добавить prelude + Result в статус
- [x] Обновить `docs/llm/stdlib.md` — добавить Result combinators
- [x] Обновить `context/PROJECT_STATE.md` — prelude, Result, error

## Verify

- [x] `cargo test` — 0 failures, 0 warnings (build)
- [x] `cargo build` — 0 warnings
