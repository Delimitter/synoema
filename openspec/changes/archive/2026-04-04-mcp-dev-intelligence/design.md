# Design: MCP Dev Intelligence

## D1: Парсер для Rust-кода — `syn` vs `tree-sitter-rust`

**Решение:** `syn` crate с features `full,visit`.

**Альтернативы:**
- `tree-sitter-rust` — C-based парсер с Rust-биндингами; мощный, но тянет C-компиляцию и увеличивает зависимости
- Regex/line-based парсинг — хрупкий, не понимает вложенные структуры
- `rust-analyzer` как библиотека — слишком тяжёлый (IDE-масштаб)

**Обоснование:** `syn` уже transitive dep через `serde_derive` → добавление как direct dep не увеличивает бинарник и не добавляет новых C-зависимостей. Парсит полный Rust syntax, включая generics, lifetimes, trait bounds. Идеален для извлечения pub API surface.

## D2: Архитектура модулей

**Решение:** 3 новых модуля в `mcp/synoema-mcp/src/`:

```
mcp/synoema-mcp/src/
  main.rs        (без изменений)
  protocol.rs    (без изменений)
  prompts.rs     (без изменений)
  resources.rs   (без изменений)
  tools.rs       (расширяется: dispatch новых tools)
  index.rs       (NEW: Live Index Engine — syn parsing, mtime cache)
  recipes.rs     (NEW: dynamic recipe generation)
  dev_tools.rs   (NEW: реализация 6 introspection tools)
```

**Альтернативы:**
- Всё в `tools.rs` — файл станет 1000+ строк, нечитаемый
- Отдельный crate `synoema-mcp-index` — overhead для 500 LOC

**Обоснование:** Один модуль = одна ответственность. `index.rs` — парсинг и кэш. `recipes.rs` — генерация рецептов. `dev_tools.rs` — tool definitions и dispatch. `tools.rs` расширяется минимально — только добавляет новые tools в list() и dispatch в call().

## D3: Кэширование — in-memory с mtime

**Решение:** `HashMap<PathBuf, CachedFile>` в `LazyLock<Mutex<...>>` (static). Проверка mtime при каждом запросе.

**Альтернативы:**
- Без кэша (parse on every request) — 80ms per request, приемлемо для 40 файлов, но расточительно
- File watcher (fsnotify) — лишняя зависимость, лишний thread
- SQLite — overkill

**Обоснование:** MCP-сервер — long-running process (запущен пока IDE открыта). In-memory кэш с mtime invalidation — оптимальный баланс: 0 доп. зависимостей, ~2ms per cached file, full rebuild ~80ms.

## D4: Token budget — ≤500 токенов на ответ

**Решение:** каждый tool ответ ограничен ~500 токенов (≈2000 символов). Truncation с маркером `... (N more items)`.

**Альтернативы:**
- Без ограничения — слабая модель (8K context) получит overflow
- Параметр `max_tokens` от клиента — слабые модели не умеют его правильно выставлять
- Pagination — сложно для слабых моделей (нужен state между вызовами)

**Обоснование:** Target — 8K context модели. System prompt ~800 tok + history ~2000 tok + tool results ~1500 tok + generation ~3000 tok. Бюджет на один tool result: ~500 tok. Фиксированный лимит проще и надёжнее параметра.

## D5: Зависимость `syn` — features

**Решение:** `syn = { version = "2", features = ["full", "visit"] }`.

**Обоснование:**
- `full` — полный парсинг Rust (нужен для fn signatures, enum variants, struct fields)
- `visit` — Visitor pattern для обхода AST без ручной рекурсии
- `syn` v2 (не v1) — serde_derive уже тянет syn, но может быть v1; явное указание v2 безопасно (Cargo допускает semver-incompatible версии параллельно)

## D6: Рецепты — какие включить

**Решение:** 4 рецепта в первой версии:
1. `add_operator` — token → scanner → parser
2. `add_builtin` — eval → codegen → runtime
3. `add_type` — types → core → eval → codegen
4. `fix_from_error` — parse error context → suggest location

**Исключены:**
- `add_crate` — слишком редкая операция
- `refactor` — слишком общий, не поддаётся шаблонизации
- `add_test` — тривиально, рецепт не нужен

**Обоснование:** 4 рецепта покрывают >80% типовых задач разработки Synoema. Каждый структурно предсказуем (enum + match block + precedence table) и хорошо ложится на AST-анализ.

## Риски

| Риск | Митигация |
|------|----------|
| `syn` не парсит файл с syntax error | Fallback на предыдущую версию из кэша + warning в ответе |
| Рецепт не находит ожидаемую структуру | Возврат ошибки с описанием что не найдено, не crash |
| `syn` v2 конфликтует с transitive v1 | Cargo разрешает обе версии параллельно |
| Ответ >500 токенов для большого crate | Truncation + `"truncated": true` в JSON |
