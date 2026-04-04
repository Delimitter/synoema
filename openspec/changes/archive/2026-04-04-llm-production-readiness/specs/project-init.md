# Spec: Project Init

## Текущее состояние

Prelude mechanism уже работает:
- `lang/prelude/prelude.sno` — содержит Result + комбинаторы
- `prepend_prelude()` в eval (lib.rs:106) и codegen (lib.rs:40) — prepend перед user source
- `error`, `env`, `env_or`, `args` — уже builtins

**Остаётся:** CLI команда `synoema init` + шаблоны + project.sno convention.

## `synoema init [name]`

### Поведение
- `synoema init myapp` → создать `myapp/`, инициализировать внутри
- `synoema init` (без имени) → инициализировать текущую директорию, имя = имя папки
- Если директория не пустая и нет `--force` → ошибка "directory not empty, use --force"

### Генерируемая структура
```
myapp/
├── project.sno          # манифест проекта
├── CLAUDE.md            # инструкции для LLM
├── src/
│   └── main.sno         # точка входа
├── tests/
│   └── main_test.sno    # пример теста
└── .gitignore           # target/, *.o
```

### project.sno (манифест)

Обычный .sno файл с string bindings:
```sno
-- Project manifest
name = "myapp"
version = "0.1.0"
entry = "src/main.sno"
```

**Парсинг:** `synoema build` парсит project.sno как программу → eval → extract string value из binding `entry`. Реализация: `eval_program()` → `env.lookup("entry")` → `Value::Str(path)`.

Это НЕ требует нового формата — используется существующий pipeline (parse → eval → extract value).

### CLAUDE.md шаблон

```markdown
# {name}

Synoema project. Language reference: https://github.com/Delimitter/synoema

## Commands
- `synoema run src/main.sno` — run (interpreter)
- `synoema jit src/main.sno` — run (JIT, native speed)
- `synoema test tests/` — run all tests
- `synoema eval "<expr>"` — evaluate expression

## Structure
- `src/` — source code, `main.sno` is entry point
- `tests/` — test files (run with `synoema test`)
- `project.sno` — project manifest (name, version, entry)

## Language Quick Ref
- Functions: `name args = body` (no def/fn/fun)
- Conditional: `? cond -> then : else` (no if/then/else)
- Lists: `[1 2 3]` (space-separated, NO commas)
- Strings: `++` for concat (not +), `"${expr}"` for interpolation
- Patterns: cons needs parens `(x:xs)`
- Error handling: `Result a e = Ok a | Err e` (in prelude)
- Pipe: `x |> f |> g`
- Types inferred, annotations optional
```

### src/main.sno шаблон
```sno
--- {name} entry point.
main = print "Hello from {name}!"
```

### tests/main_test.sno шаблон
```sno
--- Tests for {name}.

test "sanity" = 1 + 1 == 2
```

Примечание: `import "../src/main.sno"` убран из шаблона — нет гарантии что main.sno определяет что-то кроме `main`, и circular dependency с тестом.

### .gitignore шаблон
```
target/
*.o
.synoema/
```

## Реализация

В `repl/src/main.rs` добавить subcommand:
```rust
Some("init") => {
    let name = positional.get(1).map(|s| *s);
    let force = positional.iter().any(|a| *a == "--force");
    let no_git = positional.iter().any(|a| *a == "--no-git");
    init_project(name, force, no_git);
}
```

Шаблоны — `include_str!` из `lang/templates/`:
```
lang/templates/
├── main.sno.tmpl
├── test.sno.tmpl
├── project.sno.tmpl
├── CLAUDE.md.tmpl
└── gitignore.tmpl
```

Placeholder замена: `{name}` → actual name. Простой `str::replace`.

## CLI flags

| Flag | Описание |
|------|----------|
| `--force` | Инициализировать в непустой директории |
| `--no-git` | Не создавать .gitignore |

## Конвенция структуры проекта

| Элемент | Назначение |
|---------|-----------|
| `src/` | Исходный код. `main.sno` — точка входа |
| `tests/` | Тесты. Запускаются `synoema test tests/` |
| `project.sno` | Манифест: name, version, entry |
| `CLAUDE.md` | Инструкции для LLM-разработчика |
