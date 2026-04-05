# MCP Dev Intelligence

## Why

MCP-сервер Synoema сейчас предоставляет 3 инструмента (eval, typecheck, run) и статические ресурсы (language_reference, llm_ref, examples). Этого достаточно для LLM, которая **пишет** .sno код.

Но для LLM, которая **разрабатывает** сам Synoema (Rust-кодовая база, 12K LOC, 8 crates), сервер бесполезен:

1. **Нет интроспекции проекта** — LLM не может узнать структуру crates, pub API surface, зависимости между модулями без чтения каждого файла
2. **Нет поиска по коду** — нет инструмента для поиска функций, типов, паттернов в Rust-коде
3. **Нет контекста для правок** — LLM не может получить сфокусированный контекст вокруг конкретной строки (функция, типы переменных)
4. **Нет рецептов** — типовые задачи (добавить оператор, добавить builtin) требуют знания 5+ файлов в правильном порядке
5. **Ответы не оптимизированы для малого контекста** — ресурсы отдают всё целиком (~3K+ токенов), что не влезает в 8K контекст слабых Ollama-моделей (7B-14B)

## What Changes

- Добавлен **Live Index Engine** в MCP-сервер: on-demand парсинг Rust-кода через `syn`, кэш по mtime
- 6 новых MCP tools для интроспекции проекта:
  - `project_overview` — структура crates, тесты, зависимости (≤300 токенов)
  - `crate_info` — pub API surface конкретного crate (≤500 токенов)
  - `file_summary` — функции файла с сигнатурами без тел (≤300 токенов)
  - `search_code` — keyword search по коду/docs/tests (top-5, ≤400 токенов)
  - `get_context_for_edit` — сфокусированный контекст вокруг строки (≤500 токенов)
  - `recipe` — динамический пошаговый рецепт для типовых задач на основе AST-анализа
- Все ответы новых tools ограничены **≤500 токенов** для совместимости с 8K-моделями
- Новая зависимость: `syn` (уже transitive dep через serde_derive)

## Capabilities

### New Capabilities

- `mcp-project-overview`: tool для обзора структуры проекта
- `mcp-crate-info`: tool для API surface конкретного crate
- `mcp-file-summary`: tool для списка функций файла
- `mcp-search-code`: tool для поиска по кодовой базе
- `mcp-get-context`: tool для сфокусированного контекста правки
- `mcp-recipe`: tool для динамических AST-рецептов
- `mcp-live-index`: engine для on-demand парсинга Rust-кода

### Modified Capabilities

- Нет модификаций существующих — все новые tools добавляются рядом с eval/typecheck/run

## Impact

- **Код**: `mcp/synoema-mcp/src/tools.rs` (новые tools), `mcp/synoema-mcp/src/index.rs` (новый модуль — Live Index Engine), `mcp/synoema-mcp/src/recipes.rs` (новый модуль — динамические рецепты)
- **API**: 6 новых MCP tools (не ломают существующие)
- **Зависимости**: `syn` с features `full,visit` (уже transitive dep через serde_derive, добавляется как direct dep)
- **Платформы**: без изменений (pure Rust, кроссплатформенный)
