# Spec: MCP Live Index Engine

## Назначение

In-memory индекс Rust-кодовой базы Synoema, обновляемый on-demand при каждом запросе. Обеспечивает актуальные данные для всех introspection tools.

## Поведение

**WHEN** любой introspection tool вызывается
**THEN** Live Index Engine:
1. Проверяет mtime каждого `.rs` файла в `lang/crates/`
2. Если mtime изменился — перепарсивает файл через `syn`
3. Обновляет кэш в памяти
4. Возвращает данные из кэша

## Кэш

- Ключ: абсолютный путь файла
- Значение: `FileIndex { mtime, functions, structs, enums, impls, test_count, loc }`
- Инвалидация: по mtime (если файл изменился с прошлого парсинга)
- Lifetime: в памяти процесса MCP-сервера (пока сервер запущен)
- Cold start: полный парсинг всех файлов (~40 файлов, ~80ms)

## Парсинг

- Используется `syn` crate с features `full,visit`
- Извлекаются:
  - `pub fn` / `fn` с сигнатурами (имя, аргументы, return type, visibility, line number)
  - `pub struct` / `struct` с полями
  - `pub enum` / `enum` с вариантами
  - `impl` блоки с привязкой к типу
  - `#[test]` функции (для подсчёта тестов)
  - Строки кода (LOC, не считая пустые и комментарии)

## Scope

- Индексируются только файлы в `lang/crates/*/src/**/*.rs` и `lang/crates/*/tests/**/*.rs`
- MCP-код (`mcp/`) не индексируется (self-reference не нужен)
- Docs (`docs/`) индексируются отдельно (plain text search, без syn)

## Ограничения

- `syn` парсит только синтаксис — не разрешает типы, не делает inference
- Сигнатуры функций — as-is из исходного кода (не resolved types)
- Если файл содержит syntax error — пропускается с warning, используется предыдущая версия из кэша
