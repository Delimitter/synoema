# Spec: Search Early Exit

## Capability

Ранний выход из `LiveIndex::search()` и `search_in_file()` когда набрано 5 результатов.

## Current Behavior

- `collect_rs_files` обходит всё дерево, вызывая коллбэк для каждого `.rs` файла
- `search_in_file` проверяет `results.len() >= 5` на входе, но `collect_rs_files` продолжает рекурсию
- `search_in_file` вызывает `read_to_string` до проверки лимита результатов

## New Behavior

- `collect_rs_files` и `collect_files_by_ext` принимают `&Vec<SearchResult>` для проверки лимита
- Обход директории прерывается если `results.len() >= 5`
- `search_in_file` проверяет лимит ДО чтения файла

## Changes

### index.rs

1. Изменить сигнатуры `collect_rs_files` и `collect_files_by_ext`: коллбэк возвращает `bool` (true = continue, false = stop)
2. В `LiveIndex::search()` передавать ссылку на results в collect-функции через замыкание
3. В `search_in_file` переместить проверку `results.len() >= 5` перед `read_to_string`

## Acceptance Criteria

- search() прекращает обход файлов после 5 результатов
- Существующие тесты проходят
- Новый тест: search с гарантированно >5 совпадений возвращает ровно 5
