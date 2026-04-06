# Synoema

Язык программирования для LLM code generation. ~12000 LOC Rust, 998 tests, 8 crates, Cranelift JIT.

## Команды

```bash
# Из директории lang/ (workspace root)
cargo build                     # Сборка
cargo test                      # Все тесты
cargo run -p synoema-repl -- install                  # Установка в ~/.synoema/bin + PATH
cargo run -p synoema-repl -- init myapp              # Scaffold проекта
cargo run -p synoema-repl -- run examples/quicksort.sno  # Interpreter
cargo run -p synoema-repl -- jit examples/factorial.sno   # JIT
cargo run -p synoema-repl -- eval "6 * 7"                 # Eval выражения
cargo run -p synoema-repl -- build examples/quicksort.sno  # Build to bytecode
cargo run -p synoema-repl -- watch run examples/quicksort.sno  # Watch + re-run on change
cargo run -p synoema-repl -- --errors json run file.sno   # JSON ошибки
cargo run -p synoema-repl -- test examples/               # Doctests
cargo run -p synoema-repl -- doc examples/quicksort.sno   # Генерация docs
```

## Иерархия документов (по приоритету)

1. **CLAUDE.md** (этот файл) — правила и точка входа. Высший приоритет, перекрывает всё ниже
2. **Правила** — `context/RULES.md`. При конфликте с п.3–4 побеждают правила
3. **Архитектура и цели** — `context/ARCHITECTURE.md`, `context/PROJECT_STATE.md`, `context/PHASES.md`
4. **Документация OpenSpec** — specs/changes если есть. Низший приоритет, не перекрывает п.1–3

При противоречии между уровнями — верхний уровень побеждает.
**ОБЯЗАТЕЛЬНО:** перед принятием решения на основе документа — проверь, нет ли противоречия с документом выше по иерархии. Если есть — следуй верхнему. Никогда не применяй указания нижнего уровня, если они конфликтуют с верхним.

## Документация: три аудитории

| Аудитория | Документ | Назначение |
|-----------|---------|-----------|
| **Человек-пользователь** | `README.md` + `docs/LANGUAGE.md` | Quick Wins, справочник языка |
| **Контрибьютор** | `CONTRIBUTING.md` | Build, architecture, how to contribute |
| **LLM-пользователь** | `docs/llm/` | Quick reference ≤1800 токенов, compact ref ~900 токенов, 5 task-specific templates |
| **LLM-разработчик** (Claude agent) | `context/` | Rules, Architecture, Project State, Dev Guide |

Подробнее: `context/RULES.md` → секция 7.

## Навигация

| Файл | Содержимое |
|------|-----------|
| `context/RULES.md` | Правила проекта (BPE, тесты, зависимости, ABI, документация) |
| `context/ARCHITECTURE.md` | Pipeline, crates, tagged pointer ABI, FFI-паттерн |
| `context/PHASES.md` | Все завершённые фазы (9.2–23) |
| `context/PROJECT_STATE.md` | Полное состояние проекта (RU) |
| `context/DEVELOPMENT_GUIDE.md` | Как добавлять фичи, roadmap, паттерны кодирования |
| `docs/LANGUAGE.md` | Справочник языка для пользователя |
| `CONTRIBUTING.md` | Dev guide: architecture, build, tests, how to contribute |
| `docs/llm/synoema.md` | Quick reference для LLM-генерации кода (~1800 токенов) |
| `docs/llm/synoema-compact.md` | Compact reference для малых моделей 4B–32B (~900 токенов) |
| `docs/llm/templates/` | 5 task-specific prompt templates (arithmetic, lists, adt, records, string-io) |
| `docs/specs/language_reference.md` | Формальная спецификация языка |
| `docs/specs/compiler_roadmap.md` | Roadmap компилятора (фазы, архитектура) |
| `docs/research/scientific_foundations.md` | 23 научных факта |
| `docs/mcp.md` | MCP-сервер (интеграция в LLM-тулчейн) |
| `docs/articles/` | Образовательная серия (7 статей, EN+RU) |
| `docs/benchmarks.md` | Сравнительные бенчмарки: токены, runtime, LLM generation |
| `docs/testing.md` | Тестирование: 702 теста, как запускать |
| `docs/stress-server.md` | HTTP-дэшборд стресс-тестов |
| `benchmarks/` | Benchmark suite: Rust runner + Python scripts + 30 задач × 5 языков + Phase D (small models) |
| `lang/crates/` | Исходный код компилятора |

## Правила (кратко)

- Каждый оператор — ровно 1 BPE-токен (cl100k_base)
- `cargo test` чистый перед каждым коммитом (0 failures, 0 warnings)
- Новые фичи: interpreter → JIT
- Зависимости: только Cranelift + pretty_assertions
- Подробнее: `context/RULES.md`

## Статус

- 0 warnings, 0 known bugs, 998 tests
- Prelude: `lang/prelude/prelude.sno` — Result type + комбинаторы (map_ok, unwrap, is_ok, and_then и др.)
- `error : String -> a` builtin (interpreter + JIT)
- Doc Extraction API: `synoema doc --format json` + MCP tool `doc_query`
- Завершено: Phases 9.2–23 + TCO в JIT + String stdlib в JIT + Doc-as-Code + LLM Cost Reduction v1 + Region Inference + Doc Extraction API + Small Model Quality Stack Phase 1
