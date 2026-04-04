# Proposal: Record Update Syntax (`{...r, field = val}`)

## Problem Statement

Synoema поддерживает record literals (`{x = 1, y = 2}`) и field access (`r.x`), но не
поддерживает копирование записи с перезаписью отдельных полей. Это базовая операция
в функциональном программировании: вместо `{x = r.x, y = r.y, z = 42}` (многословно)
должен работать `{...r, z = 42}` (лаконично).

LLM при генерации кода на Synoema не может выразить immutable update pattern — один
из ключевых idioms для работы с иммутабельными data structures.

## Scope

- **Лексер**: новый токен `Token::DotDotDot` (`...`) — 1 BPE-токен на cl100k_base
- **Парсер**: новый `ExprKind::RecordUpdate { base: Box<Expr>, updates: Vec<(String, Expr)> }`
- **Типизация**: вывод типа для RecordUpdate через row-polymorphism (base должен быть record, updates должны совпадать с существующими полями)
- **Core IR**: desugar RecordUpdate → `let __base = r in {field1 = __base.field1, ..., fieldN = val}`
- **Eval**: eval RecordUpdate — копирование полей из base + override
- **JIT**: compile RecordUpdate через существующие `synoema_record_new/set/get`
- **Тесты**: interpreter + JIT + type errors
- **GBNF**: добавить record-update rule в `tools/constrained/synoema.gbnf`

## What is NOT in scope

- Nested update: `{...r, a.b = 5}` — слишком сложно для alpha
- Field deletion: `{...r, -field}` — нет use case сейчас
- Two-record merge: `{...r1, ...r2}` — неоднозначно при пересечении ключей
- Record extension (добавление нового поля): `{...r, newField = x}` — ошибка типов

## Success Criteria

1. `{...{x=1,y=2}, x=10} == {x=10, y=2}` — работает в interpreter
2. `{...{x=1,y=2}, x=10} == {x=10, y=2}` — работает в JIT
3. `{...r, nonexistent = 1}` → type error
4. BPE: `...` = 1 токен на cl100k_base
5. `cargo test` — 0 failures, 0 warnings
6. Документация обновлена: `docs/llm/synoema.md`, `docs/llm/stdlib.md`
