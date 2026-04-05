# Spec: doc_query Serde JSON

## Capability

Замена ручного JSON-экранирования в `doc_query` на `serde_json`.

## Current Behavior

- `doc_query` строит JSON-строку через ручной `format!()` + closure `esc()` (4 replace вызова)
- Риск некорректного JSON при спецсимволах
- O(n_decls * avg_doc_length) string allocations

## New Behavior

- Структуры `DocQueryResult`, `DocFunction`, `DocType`, `DocModule`, `DocExample` с `#[derive(Serialize)]`
- Финальный вывод через `serde_json::to_string()`
- Truncation по длине JSON-строки после сериализации

## Changes

### dev_tools.rs

1. Добавить serde structs для doc_query выхода
2. Заменить ручное построение JSON на заполнение структур + `serde_json::to_string()`
3. Удалить closure `esc`

## Acceptance Criteria

- `doc_query` возвращает валидный JSON (проверяется serde_json::from_str в тесте)
- Существующий тест `doc_query_valid_file` проходит
- Новый тест: doc_query с спецсимволами (кавычки, backslash) в комментариях → корректный JSON
