# Proposal: Fix Env Clone Explosion in Interpreter

## Problem

`synoema eval "6 * 7"` потребляет 106 ГБ памяти и убивается OOM killer. Причина — экспоненциальное клонирование `Env` при регистрации функций в `eval_program()`.

### Механизм утечки

`Value::Func` содержит `env: Env`. `Env` содержит `HashMap<String, Value>`. При `#[derive(Clone)]` на обоих — deep copy всего дерева.

В `eval_program()` (eval.rs:134-191):
1. **Pass 1** (строки 134-148): для каждой функции `env.clone()` клонирует весь env, включая ранее зарегистрированные функции с их вложенными env
2. **Pass 2** (строки 160-175): `snapshot = env.clone()` + `snapshot.clone()` для каждой функции — повторное клонирование всего

При N=60 (40 builtins + 20 prelude функций), каждый `env.clone()` рекурсивно копирует все вложенные env → геометрическая прогрессия → 100+ ГБ.

### Вторичная проблема

`Box::leak()` в runtime.rs:575 и compiler.rs:573 — каждая строка в JIT навечно аллоцируется. Minor по сравнению с Env, но тоже утечка.

## Solution

Заменить `env: Env` на `env: Rc<Env>` в `Value::Func` и `Value::Closure`. `Rc::clone()` = инкремент счётчика (O(1)) вместо deep copy (O(N!)).

Evaluator однопоточный → `Rc` достаточно (не `Arc`). Для мутации: `Env` уже использует `push_scope`/`pop_scope` — child environments создаются через `child()` → clone + push. С `Rc` нужен `Rc::make_mut` или explicit clone-on-write при создании child scope.

## Scope

| # | Что | Crate |
|---|-----|-------|
| 1 | `Value::Func { env: Rc<Env> }`, `Value::Closure { env: Rc<Env> }` | synoema-eval |
| 2 | Обновить `eval_program()` — использовать `Rc::new()` и `Rc::clone()` | synoema-eval |
| 3 | Обновить все `env.clone()` → `Rc::clone(&env)` или explicit clone для mutation | synoema-eval |
| 4 | Обновить `child()` — clone inner Env, wrap in new Rc | synoema-eval |
| 5 | Заменить `Box::leak` на arena allocation в compiler.rs и runtime.rs | synoema-codegen |
| 6 | Тесты на отсутствие регрессии + новый тест на размер памяти | all |

## Non-goals

- Не менять архитектуру Env (scope chain остаётся)
- Не менять JIT pipeline (Cranelift)
- Не вводить GC или tracing
- Не менять публичный API

## Success Criteria

- `synoema eval "6 * 7"` завершается за <1s с <50 МБ памяти
- `cargo test` — 0 failures, 0 warnings
- Все 890+ тестов проходят
- Box::leak заменён на arena allocation
