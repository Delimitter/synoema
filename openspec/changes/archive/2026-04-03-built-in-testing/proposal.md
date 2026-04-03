# Proposal: Built-in Testing — test declarations + property-based testing

## Problem Statement

Synoema имеет только doctests (`--- example: expr == val`). Они привязаны к отдельным функциям и не позволяют:
1. Тестировать композицию нескольких функций (standalone тесты)
2. Описывать инварианты — свойства, верные для ВСЕХ входов (property-based testing)
3. Фильтровать/запускать тесты по имени

Property-based testing (QuickCheck) — идиома, рождённая в функциональном программировании. Чистые функции = детерминированное отображение input→output, что делает автогенерацию входов естественной и безопасной.

## Scope

- Новый keyword `test` — top-level декларация: `test "name" = <Bool-expr>`
- Новый keyword `prop` — генератор в test-выражениях: `prop xs -> expr` (type-driven)
- Оператор `implies` для conditional properties: `cond implies body`
- Расширение `synoema test` — запуск test-деклараций + doctests, фильтр `--filter`
- Поддержка в interpreter. JIT — вне scope (тесты не performance-critical)

## What is NOT in scope

- `assert` (imperative side-effect, не ложится на чистую парадигму)
- Custom generators (хватает type-driven)
- Mocks, fixtures, setup/teardown (нет state)
- Snapshot/expect тесты
- JIT-компиляция тестов

## Success Criteria

- `test "name" = expr` парсится, выполняется, отображается в `synoema test`
- `prop xs -> expr` генерирует 100 случайных входов по типу переменной
- `implies` отбрасывает невалидные входы без провала
- `--filter "name"` фильтрует тесты по подстроке имени
- Все 771+ существующих теста зелёные, 0 warnings
- Новые тесты покрывают каждую конструкцию (positive + negative)
- Документация обновлена: llm/, specs/, testing.md
