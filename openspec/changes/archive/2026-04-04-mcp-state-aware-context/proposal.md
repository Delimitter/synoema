# MCP State-Aware Context

## Why

LLM, работающая с Synoema через MCP-сервер, всегда получает одинаковый контекст вне зависимости от того, что она делает — пишет новый код, отлаживает ошибку типов или анализирует runtime. Это расходует контекстное окно: при отладке TypeError не нужен полный stdlib catalog, а при написании нового кода не нужны arena stats.

State-aware context решает это: MCP-сервер отслеживает текущее состояние разработки по вызовам инструментов и предоставляет baseline контекст, релевантный именно этому состоянию.

## What Changes

- Новый модуль `state.rs` в `synoema-mcp`: enum `AppState` (Create/Check/Run/Debug), `StateTracker` с переходами по tool calls
- Новый инструмент `get_context`: возвращает baseline контекст для текущего состояния
- Новый инструмент `get_state`: возвращает текущее состояние (для отладки/прозрачности)
- Интеграция трекера в `handle_tools_call`: каждый вызов инструмента обновляет состояние

## Capabilities

### New Capabilities

- `mcp-state-tracker`: определение текущего состояния разработки по вызовам инструментов
- `mcp-context-tool`: инструмент `get_context`, возвращающий state-aware baseline

### Modified Capabilities

- Все существующие tool calls теперь обновляют внутренний StateTracker (прозрачно, без изменения API)

## Impact

- **Код**: `mcp/synoema-mcp/src/state.rs` (новый), `mcp/synoema-mcp/src/tools.rs` (интеграция), `mcp/synoema-mcp/src/main.rs` (трекер в loop)
- **API**: 2 новых MCP tool (`get_context`, `get_state`), 0 breaking changes
- **Зависимости**: нет новых (std only)
- **Тесты**: ~15 новых тестов (state transitions + context content)

## Scope

- Только MCP-сервер (`mcp/synoema-mcp/`). Ядро компилятора (`lang/`) не затрагивается.
- ~200 LOC нового кода.
