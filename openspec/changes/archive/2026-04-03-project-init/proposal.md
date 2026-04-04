# Proposal: project-init — `synoema init` scaffolding command

## Problem Statement

LLM не может инициализировать Synoema-проект с нуля без внешних знаний о структуре директорий, имени entry point и формате манифеста. Отсутствие `synoema init` вынуждает LLM угадывать конвенции или пользователя — создавать структуру вручную.

Текущий flow:
```
[USER] "create project" ──→ ? (no convention) ──→ [LLM] guesses layout ──→ [ERROR]
```

Target flow:
```
[USER] "create project" ──→ synoema init myapp ──→ ready-to-run project
```

## Scope

### Что входит

1. **`synoema init [name]` subcommand** в `repl/src/main.rs`
   - `synoema init myapp` — создаёт директорию `myapp/` с полной структурой
   - `synoema init` (без имени) — инициализирует текущую директорию
   - `synoema init --force` — перезаписывает существующую структуру
   - `synoema init --no-git` — без `.gitignore`

2. **Шаблоны** в `lang/templates/` (embed via `include_str!`):
   - `main.sno.tmpl` — hello world на Synoema
   - `test.sno.tmpl` — пример теста с doctest
   - `project.sno.tmpl` — манифест `{name = "...", version = "0.1.0", entry = "src/main.sno"}`
   - `CLAUDE.md.tmpl` — LLM context file (auto-context для Claude)
   - `gitignore.tmpl` — стандартный `.gitignore`

3. **Структура созданного проекта:**
   ```
   myapp/
   ├── src/
   │   └── main.sno        ← entry point
   ├── tests/
   │   └── test.sno        ← тесты
   ├── project.sno         ← манифест
   ├── CLAUDE.md           ← LLM context
   └── .gitignore
   ```

4. **Обновление CLI help** — добавить `synoema init` в `--help`

5. **Обновление документации** — `CLAUDE.md`, `docs/user/README.md`, `context/PROJECT_STATE.md`

### Что НЕ входит

- `synoema build` (separate change)
- Prelude mechanism (separate change — llm-production-readiness Phase A1-A3)
- REPL изменения
- Форматирование кода в шаблонах (шаблоны уже форматированы)
- Валидация имени проекта (разрешены любые FS-safe имена)

## Success Criteria

1. `synoema init myapp && cd myapp && synoema run src/main.sno` — выводит `"Hello, myapp!"`
2. `synoema init` без имени — инициализирует текущую директорию
3. `synoema init` в непустой директории без `--force` → понятная ошибка
4. `synoema init --no-git` — не создаёт `.gitignore`
5. `synoema init --help` показывает новую команду
6. `cargo test` — 0 failures, 0 warnings после изменений
7. Сгенерированный `src/main.sno` компилируется без ошибок
8. Сгенерированный `tests/test.sno` проходит `synoema test`
