# Synoema — Руководство пользователя

Документация для программиста, пишущего `.sno` код.

> Если вы языковая модель, генерирующая `.sno` код — используйте [`docs/llm/synoema.md`](../llm/synoema.md).

## Содержимое

| Файл | Содержимое |
|------|-----------|
| [`../install.md`](../install.md) | Установка компилятора и MCP-сервера |
| [`../versioning.md`](../versioning.md) | Политика версионирования |
| `quickstart.md` | Первые шаги — первая программа (TODO) |
| `syntax.md` | Языковые конструкции с объяснениями (TODO) |
| `examples.md` | Примеры программ с разбором (TODO) |

## Быстрый старт

Установка: см. [`docs/install.md`](../install.md).

Запуск:

```bash
cd synoema/lang

# Запустить файл (интерпретатор)
cargo run -p synoema-repl -- run examples/quicksort.sno

# Запустить файл (JIT)
cargo run -p synoema-repl -- jit examples/factorial.sno

# Вычислить выражение
cargo run -p synoema-repl -- eval "6 * 7"
```

## Ключевые отличия от Python/Haskell

| Python/Haskell | Synoema |
|----------------|---------|
| `def f(x):` | `f x =` |
| `if cond: ... else: ...` | `? cond -> then : else` |
| `[1, 2, 3]` | `[1 2 3]` (пробел вместо запятой) |
| `let x = ...` | отступ (offside rule) |
| `return x` | нет `return` — последнее выражение |

## Полная документация

- [Установка](../install.md) — бинарники, cargo install, MCP-сервер
- [Версионирование](../versioning.md) — политика совместимости
- [Формальная спецификация языка](../specs/language_reference.md)
- [Научные основания](../research/scientific_foundations.md)
- [Примеры](../../lang/examples/)
- [MCP-интеграция](../mcp.md) — использование Synoema через Claude Desktop / Cursor
