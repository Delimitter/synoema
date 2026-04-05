# Tasks: MCP Performance Improvements

## 1. Search early exit (`index.rs`)

- [x] 1.1 Изменить `collect_rs_files`: коллбэк `FnMut(&Path) -> bool`, рекурсия прерывается при false
- [x] 1.2 Изменить `collect_files_by_ext`: аналогично — коллбэк возвращает bool
- [x] 1.3 В `search_in_file`: переместить проверку `results.len() >= 5` ПЕРЕД `read_to_string`
- [x] 1.4 В `LiveIndex::search()`: замыкания в collect_* возвращают `results.len() < 5`
- [x] 1.5 Добавить тест: search с запросом, гарантирующим >5 совпадений, возвращает ровно 5

## 2. Cache all_crates (`index.rs`)

- [x] 2.1 Добавить поле `crates_cache: Mutex<Option<(SystemTime, Vec<CrateSummary>)>>` в `LiveIndex`
- [x] 2.2 Обновить `INDEX` static: инициализировать `crates_cache: Mutex::new(None)`
- [x] 2.3 В `all_crates()`: проверить mtime `lang/crates/`, при совпадении — вернуть кэш
- [x] 2.4 Добавить тест: повторный вызов `all_crates()` возвращает тот же результат (косвенная проверка кэша)

## 3. doc_query serde (`dev_tools.rs`)

- [x] 3.1 Добавить `use serde::Serialize;` и serde structs: `DocQueryResult`, `DocFunction`, `DocType`, `DocModule`, `DocExample`
- [x] 3.2 Переписать `tool_doc_query`: заполнять структуры вместо ручного format!
- [x] 3.3 Финальный вывод через `serde_json::to_string(&result).unwrap_or_default()`
- [x] 3.4 Удалить closure `esc`
- [x] 3.5 Добавить тест: doc_query результат парсится serde_json::from_str как Value без ошибок
- [x] 3.6 Проверить существующий тест `doc_query_valid_file` — должен проходить

## 4. State error cap (`state.rs`)

- [x] 4.1 Добавить `const MAX_ERROR_LEN: usize = 500;`
- [x] 4.2 В `on_tool_result`: обрезать error_text до MAX_ERROR_LEN при записи в last_error
- [x] 4.3 Добавить тест: ошибка >500 символов обрезается до 500 + маркер

## 5. Cargo test clean

- [x] 5.1 `cargo test` в `mcp/` — 0 failures
- [x] 5.2 `cargo clippy -p synoema-mcp` — 0 warnings
