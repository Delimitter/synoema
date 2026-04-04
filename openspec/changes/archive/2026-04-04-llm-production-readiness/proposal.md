# Proposal: LLM Production Readiness — полная готовность к разработке приложений

## Problem Statement

Synoema имеет сильный core language (types, ADTs, records, modules, closures, pattern matching) и отличный verification pipeline (MCP, JSON errors, GBNF, feedback loop). Но есть structural gap: LLM не может **инициализировать проект с нуля** и **написать нетривиальное приложение** без ряда отсутствующих компонентов.

Текущий flow LLM:
```
[CONTEXT] ──✅──▶ [INIT] ──❌──▶ [CODE] ──⚠️──▶ [VERIFY] ──✅──▶ [SCALE] ──❌──▶
```

Gaps обнаружены в 3 слоях:
1. **Project Init** — нет scaffolding, нет конвенции структуры, нет манифеста
2. **Язык/Stdlib** — нет error handling (Result), нет Map/Dict, нет record update, нет JSON parse, нет env/args
3. **Tooling** — нет formatter, нет build multi-file, VS Code extension пропала

## Scope

### Слой 1: Project Init (критичный)
- `synoema init [name]` — scaffolding команда
- Конвенция структуры проекта (`src/`, `tests/`, `main.sno`)
- Шаблон CLAUDE.md для LLM-проектов (автоконтекст)
- Минимальный `project.sno` манифест (name, entry, version)

### Слой 2: Язык — stdlib gaps (критичный)
- **Result type** — `Result a e = Ok a | Err e` + стандартные комбинаторы (map_ok, unwrap, etc.)
- **Map type** — `Map k v` через sorted association list (не HashMap, соблюдаем минимализм)
- **Record update syntax** — `{...r, x = 42}` spread operator
- **JSON parsing** — `json_parse : String -> Result JsonValue String`
- **Env vars** — `env : String -> String` (interpreter)
- **CLI args** — `args : [String]` (interpreter)

### Слой 3: Tooling
- **`synoema fmt`** — canonical formatter (2-space indent, sorted imports)
- **`synoema build`** — multi-file compilation с entry point из project.sno
- **VS Code extension** — TextMate grammar для .sno (восстановить)

## What is NOT in scope

- Пакетный менеджер / registry (слишком рано для alpha)
- LSP (MCP покрывает LLM-case)
- Async/await (Synoema — strict, eager)
- JIT для Map/JSON/env/args (interpreter-first)
- Date/Time stdlib
- HTTP client (tcp_listen/accept покрывает серверный case)
- Random stdlib (property tests достаточно)

## Success Criteria

1. `synoema init myapp && cd myapp && synoema run src/main.sno` — работает
2. LLM видит CLAUDE.md в проекте и знает как писать код
3. Error handling через Result: `parse_int "42" |> map_ok (\n -> n * 2)` — компилируется
4. `Map` достаточен для key-value storage в типичных приложениях
5. `{...record, field = val}` — работает в interpreter + JIT
6. `synoema fmt file.sno` — идемпотентный форматирование
7. Все существующие 864 тестов зелёные, 0 warnings
8. Каждый новый keyword — 1 BPE-токен
