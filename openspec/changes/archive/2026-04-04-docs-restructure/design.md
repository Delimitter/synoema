---
id: design
type: design
status: done
---

# Design: Documentation Restructure

## Architecture Decision: Information Flow

```
  Новый пользователь ──→ README.md (root)
                          │
           ┌──────────────┼──────────────┐
           ▼              ▼              ▼
    docs/LANGUAGE.md   Quick Wins    CONTRIBUTING.md
    "выучить язык"     (в README)    "собрать и помочь"
           │                              │
           └──────→ docs/install.md ◀─────┘
                   (единый источник)
```

Принцип: **каждый файл отвечает на ОДИН вопрос**. Никакой файл не пытается быть всем.

## README.md — Quick Wins формат

### Структура

1. **Header** — название + одна строка "что это" + version badge
2. **30 Seconds** — `synoema eval "6 * 7"` → 42 (буквально одна команда)
3. **Install** — три варианта по 2 строки каждый (cargo install / binary / npx), ссылка на docs/install.md для деталей
4. **Quick Wins** — 5 сценариев, каждый в 3-4 строки:
   - Запустить программу (`synoema run`)
   - JIT-компиляция (`synoema jit`)
   - Scaffold проекта (`synoema init`)
   - Встроить в LLM-пайплайн (`npx synoema-mcp`)
   - VS Code (`Cmd+Shift+R`)
5. **Show Me The Code** — 15-20 строк лучших примеров (не 60 как сейчас)
6. **Why Synoema** — бенчмарки (таблица токенов + производительность), сжатые
7. **Links** — 5 ссылок: Language, Contributing, LLM Ref, Specs, Research

### Что убираем из README

| Секция | Куда переезжает |
|--------|----------------|
| Language Reference таблица (25 строк) | docs/LANGUAGE.md |
| Interpreter vs JIT таблица | docs/LANGUAGE.md |
| Architecture диаграмма + crates таблица | CONTRIBUTING.md |
| Constrained Decoding (15 строк) | docs/LANGUAGE.md (секция LLM Integration) |
| Structured Diagnostics | docs/LANGUAGE.md |
| Scientific Foundations (6 ссылок) | ссылка на docs/research/ |
| Roadmap (30 строк) | CONTRIBUTING.md |

## docs/LANGUAGE.md — Справочник языка

### Принципы
- Для **человека** (с объяснениями "почему"), не для LLM
- Один документ, не 4 разрозненных
- Каждая конструкция — объяснение + runnable пример из lang/examples/
- **Не дублирует** docs/llm/ (та оптимизирована по токенам) и docs/specs/ (та формальная)

### Структура

1. **Mental model** — 5 правил ("нет def", "нет return", пробелы вместо запятых...)
2. **Функции и паттерн-матчинг** — определения, multi-equation, wildcard, cons patterns
3. **Типы данных** — Int, Float, Bool, String, List, Record, ADT
4. **Операторы** — таблица приоритетов с пояснениями
5. **Контроль потока** — ternary `?->:`, local bindings, pipe `|>`
6. **Коллекции** — списки, ranges, comprehensions
7. **Записи** — литералы, punning, destructuring, update
8. **Алгебраические типы** — ADT, pattern matching, derive
9. **Тайп-классы** — trait/impl с примерами
10. **Модули** — mod/use, multi-file import
11. **IO и эффекты** — print, readline, bind, sequencing
12. **Стандартная библиотека** — полная таблица из stdlib
13. **Тестирование** — doctest, unit, property
14. **Обработка ошибок** — Result type, error, комбинаторы
15. **Интеграция с LLM** — constrained decoding, JSON errors, MCP

### Источники контента
- Syntax таблица из текущего README.md
- Примеры из lang/examples/
- Объяснения из docs/llm/synoema.md (развёрнутые для человека)
- Stdlib из docs/llm/stdlib.md
- Prelude из lang/prelude/prelude.sno

## CONTRIBUTING.md — Developer Guide

### Структура

1. **Prerequisites** — Rust ≥1.75, cargo, git
2. **Quick Start** — clone → build → test (3 команды)
3. **Architecture** — pipeline diagram + crates table (из текущего README)
4. **Project Structure** — дерево директорий с пояснениями
5. **Running Tests** — cargo test, per-crate, stress tests, doctests
6. **Adding a Feature** — interpreter-first → JIT, BPE check, test-first
7. **Code Style** — идиоматический Rust, minimal deps, no unsafe кроме FFI
8. **Releases** — как собрать бинарники, platform-specific
9. **Licensing** — multi-license summary, SPDX headers, DCO
10. **Roadmap** — что ещё не сделано

## lang/README.md — Минимальный указатель

Заменить 359-строчную копию на:
```markdown
# Synoema — Compiler Source

This is the Rust workspace for the Synoema compiler.

See the [main README](../README.md) for usage, or [CONTRIBUTING.md](../CONTRIBUTING.md) for development.
```

## docs/user/README.md — Удалить

Роль "точки входа для пользователя" переходит к root README.md. Дублирование убираем.

## Языковые решения

- **README.md** — английский (стандарт для GitHub, международная аудитория)
- **docs/LANGUAGE.md** — английский (язык программирования, код на английском)
- **CONTRIBUTING.md** — английский (международные контрибьюторы)
- Комментарии в коде и CLAUDE.md — без изменений (русский)
