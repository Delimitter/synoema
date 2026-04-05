# MCP Performance Improvements

## Why

MCP-сервер Synoema имеет несколько узких мест, влияющих на latency tool calls и качество ответов при подключении к LLM:

1. **search() не прерывается после 5 результатов на уровне файлового обхода** — `collect_rs_files` и `collect_files_by_ext` обходят ВСЁ дерево файлов, даже когда 5 результатов уже найдено. Проверка `results.len() >= 5` есть только внутри `search_in_file`, но коллбэк продолжает вызываться для каждого файла
2. **all_crates() пересканирует все крейты при каждом вызове** — нет кэша на уровне директории `lang/crates/`. Каждый вызов `project_overview` делает full scan
3. **doc_query строит JSON вручную** через `replace()` × 4 на каждую декларацию вместо serde_json. Лишние аллокации, риск некорректного JSON при спецсимволах
4. **state.rs: last_error не ограничен** — длинные stack trace хранятся полностью, раздувая baseline_context
5. **Рецепты делают 3-5 отдельных get_file() вызовов** — каждый шаг независимо запрашивает индекс, нет batch loading
6. **search_in_file читает весь файл даже если results уже полон** — `std::fs::read_to_string` вызывается до проверки лимита

## What Changes

- Ранний выход из search() на уровне файлового обхода (передача `&mut results` с проверкой лимита)
- Кэш all_crates() с инвалидацией по mtime директории `lang/crates/`
- Замена ручного JSON-экранирования на `serde_json::to_string()` в doc_query
- Ограничение last_error до 500 символов в state.rs
- Ранний выход из search_in_file до чтения файла (проверка results.len() перед read_to_string)

## Capabilities

### Modified Capabilities

- `mcp-live-index`: search() с ранним выходом + кэш all_crates() + оптимизация search_in_file
- `mcp-state-tracker`: ограничение last_error
- `mcp-doc-query`: JSON через serde вместо ручного экранирования

## Impact

- **Код**: `mcp/synoema-mcp/src/index.rs` (search + all_crates), `mcp/synoema-mcp/src/state.rs` (last_error cap), `mcp/synoema-mcp/src/dev_tools.rs` (doc_query serde)
- **API**: без изменений — все tools сохраняют существующие интерфейсы
- **Зависимости**: без изменений (serde_json уже в зависимостях)
- **Тесты**: новые тесты для early exit, кэша all_crates, serde doc_query
