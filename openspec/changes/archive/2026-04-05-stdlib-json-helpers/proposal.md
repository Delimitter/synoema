# Proposal: stdlib-json-helpers

## Problem

bench-viz-v6 показал ratio 2.05x (Synoema дороже Python). Декомпозиция: 254 из 428 токенов (59%) — boilerplate, который в Python бесплатен. Без boilerplate Synoema 0.83x (дешевле Python).

Boilerplate:
- 4 JSON-экстрактора (json_str, json_int, json_arr, json_obj) — ~100 tok
- join через рекурсию (intercalate) — ~40 tok
- for_each через foldl хак — ~15 tok
- lookup_obj (ненужен с починенным json_get) — ~50 tok (документация)

## Scope

1. Добавить в prelude: `json_str`, `json_int`, `json_arr`, `json_obj`, `intercalate`, `for_each`
2. Обновить docs/llm/stdlib.md — описать новые функции
3. Обновить docs/llm/synoema.md — добавить примеры JSON-пайплайна и sequencing
4. Обновить AGENTS.md.tmpl — добавить новые функции в Stdlib section
5. Тесты для новых stdlib-функций

## Non-goals

- Не менять синтаксис языка (`;` в let-блоках, `_ = expr`)
- Не менять json_get API
- Не менять парсер
