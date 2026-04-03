# Synoema

Язык программирования для LLM code generation. ~12000 LOC Rust, 634 tests, 8 crates, Cranelift JIT.

## Команды

```bash
# Из директории lang/ (workspace root)
cargo build                     # Сборка
cargo test                      # Все тесты
cargo run -p synoema-repl -- run examples/quicksort.sno  # Interpreter
cargo run -p synoema-repl -- jit examples/factorial.sno   # JIT
cargo run -p synoema-repl -- eval "6 * 7"                 # Eval выражения
cargo run -p synoema-repl -- --errors json run file.sno   # JSON ошибки
```

## Иерархия документов (по приоритету)

1. **CLAUDE.md** (этот файл) — правила и точка входа. Высший приоритет, перекрывает всё ниже
2. **Правила** — `context/RULES.md`. При конфликте с п.3–4 побеждают правила
3. **Архитектура и цели** — `context/ARCHITECTURE.md`, `context/PROJECT_STATE.md`, `context/PHASES.md`
4. **Документация OpenSpec** — specs/changes если есть. Низший приоритет, не перекрывает п.1–3

При противоречии между уровнями — верхний уровень побеждает.
**ОБЯЗАТЕЛЬНО:** перед принятием решения на основе документа — проверь, нет ли противоречия с документом выше по иерархии. Если есть — следуй верхнему. Никогда не применяй указания нижнего уровня, если они конфликтуют с верхним.

## Документация: три аудитории

| Аудитория | Директория | Назначение |
|-----------|-----------|-----------|
| **Человек-пользователь** | `docs/user/` | Туториалы, синтаксис, примеры |
| **LLM-пользователь** | `docs/llm/` | Quick reference ≤1800 токенов, таблицы, аксиомы |
| **LLM-разработчик** (Claude agent) | `context/` | Rules, Architecture, Project State, Dev Guide |

Подробнее: `context/RULES.md` → секция 7.

## Навигация

| Файл | Содержимое |
|------|-----------|
| `context/RULES.md` | Правила проекта (BPE, тесты, зависимости, ABI, документация) |
| `context/ARCHITECTURE.md` | Pipeline, crates, tagged pointer ABI, FFI-паттерн |
| `context/PHASES.md` | Все завершённые фазы (9.2–18) |
| `context/PROJECT_STATE.md` | Полное состояние проекта (RU) |
| `context/DEVELOPMENT_GUIDE.md` | Как добавлять фичи, roadmap, паттерны кодирования |
| `docs/user/README.md` | Точка входа для человека-пользователя |
| `docs/llm/synoema.md` | Quick reference для LLM-генерации кода |
| `docs/specs/language_reference.md` | Формальная спецификация языка |
| `docs/specs/compiler_roadmap.md` | Roadmap компилятора (фазы, архитектура) |
| `docs/research/scientific_foundations.md` | 23 научных факта |
| `docs/mcp.md` | MCP-сервер (интеграция в LLM-тулчейн) |
| `docs/articles/` | Образовательная серия (7 статей, EN+RU) |
| `docs/testing.md` | Тестирование: 634 теста, как запускать |
| `docs/stress-server.md` | HTTP-дэшборд стресс-тестов |
| `lang/crates/` | Исходный код компилятора |

## Правила (кратко)

- Каждый оператор — ровно 1 BPE-токен (cl100k_base)
- `cargo test` чистый перед каждым коммитом (0 failures, 0 warnings)
- Новые фичи: interpreter → JIT
- Зависимости: только Cranelift + pretty_assertions
- Подробнее: `context/RULES.md`

## Статус

- 0 warnings, 0 known bugs, 634/634 tests
- Текущий приоритет: Phase 18 (сетевые примитивы + строковый stdlib) и расширение диагностики
