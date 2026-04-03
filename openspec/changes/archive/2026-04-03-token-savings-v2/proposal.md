# Proposal: Token Savings v2

## Problem Statement

Synoema достигла -46% токенов vs Python. Две синтаксические конструкции создают избыточные токены:

1. **Record literals** — `{x = x, y = y}` вместо `{x, y}` при совпадении имени поля и переменной
2. **Wildcard import** — `use Math (square cube abs)` вместо `use Math (*)` при использовании всех экспортов

String interpolation уже реализована (Phase archive 2026-04-03).

## Scope

### Feature 1: Record Punning

Синтаксис: `{x, y}` эквивалентно `{x = x, y = y}`.

- Только parser-level desugar
- Смешанный стиль: `{x, y, z = x + y}`
- Экономия: ~60% токенов на record literal с совпадающими именами

### Feature 2: Wildcard Import

Синтаксис: `use Math (*)` эквивалентно `use Math (all_exported_names)`.

- Parser + eval + codegen изменения (resolver)
- Оба варианта сохраняются: selective (`use Math (square)`) + wildcard (`use Math (*)`)
- Экономия: линейная по количеству экспортов

## Success Criteria

- `{x, y}` десахарится в `{x = x, y = y}` на уровне парсера
- `use Math (*)` резолвит все имена модуля в interpreter и JIT
- Все тесты зелёные, 0 warnings
- GBNF, docs/llm/, language_reference обновлены
- BPE-alignment: `*` = 1 токен (cl100k_base: 9), `{` `}` `,` = 1 токен каждый ✓
