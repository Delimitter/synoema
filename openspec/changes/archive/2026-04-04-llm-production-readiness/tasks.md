# Tasks: LLM Production Readiness

> Dependency graph: A → B/C → D → E/F (see design.md)
> Каждая Phase — независимо коммитится. Внутри Phase — порядок обязателен.

---

## Phase A: Project Init + Prelude + Result

### A1. Prelude mechanism
- [x] Создать `lang/prelude/prelude.sno` — пустой файл (placeholder)
- [x] В `eval.rs`: `load_prelude()` — parse + eval prelude перед пользовательской программой
- [x] `include_str!("../../prelude/prelude.sno")` — prelude встроен в бинарник
- [x] Тесты: prelude functions доступны в пользовательском коде без import
- [x] Prelude не ломает существующие 864 теста

### A2. `error` function
- [x] В `eval.rs`: builtin `error : String -> a` — panic с сообщением
- [x] В JIT (`runtime.rs`): `synoema_error(msg_ptr: i64) -> !` — FFI
- [x] Тесты: `error "boom"` → runtime panic с "boom"

### A3. Result type (в prelude)
- [x] Добавить в `prelude.sno`: `Result a e = Ok a | Err e`
- [x] Добавить комбинаторы: `map_ok`, `map_err`, `unwrap`, `unwrap_or`, `is_ok`, `is_err`, `and_then`
- [x] Тесты: `map_ok (\x -> x + 1) (Ok 5) == Ok 6`
- [x] Тесты: `unwrap_or 0 (Err "fail") == 0`
- [x] Тесты: `and_then (\x -> Ok (x * 2)) (Ok 5) == Ok 10`
- [x] Тесты: pipe chain — `Ok 5 |> map_ok (\n -> n * 2) |> unwrap_or 0` == 10

### A4. `synoema init`
- [x] Создать `lang/templates/main.sno.tmpl` — шаблон с placeholder для имени
- [x] Создать `lang/templates/test.sno.tmpl` — шаблон теста
- [x] Создать `lang/templates/project.sno.tmpl` — шаблон манифеста
- [x] Создать `lang/templates/CLAUDE.md.tmpl` — шаблон для LLM
- [x] Создать `lang/templates/gitignore.tmpl` — .gitignore
- [x] В `repl/src/main.rs`: добавить `Some("init")` subcommand
- [x] `synoema init myapp` — создаёт структуру `myapp/src/`, `myapp/tests/`, etc.
- [x] `synoema init` (без имени) — инициализирует текущую директорию
- [x] `synoema init` в непустой директории → ошибка (без --force)
- [x] `synoema init --force` → инициализирует даже в непустой
- [x] `synoema init --no-git` → не создаёт .gitignore
- [x] Тесты: init creates correct directory structure
- [x] Тесты: generated `src/main.sno` компилируется: `synoema run`

### A5. Documentation update (Phase A)
- [x] Обновить `CLAUDE.md` — добавить `synoema init` в команды
- [x] Обновить `docs/user/README.md` — добавить quickstart с init
- [x] Обновить `docs/llm/synoema.md` — добавить секцию Result
- [x] Обновить `docs/llm/stdlib.md` — добавить Result combinators
- [x] Обновить `context/PROJECT_STATE.md` — prelude, Result, init
- [x] Обновить `--help` output — добавить init

---

## Phase B: Record Update + Env/Args

### B1. Record update syntax (`{...r, field = val}`)
- [x] Lexer: добавить `Token::DotDotDot` (`...`), BPE-verify = 1 token
- [x] Parser: `ExprKind::RecordUpdate { base, updates }` — парсить `{...expr, field = val}`
- [x] Types: infer RecordUpdate — base must be record, updates must match fields
- [x] Core IR: desugar RecordUpdate → field extraction + new record
- [x] Eval: eval RecordUpdate — copy fields, apply overrides
- [x] JIT: compile RecordUpdate — allocate new RecordNode, copy + override
- [x] Тесты interpreter: `{...{x=1, y=2}, x=10} == {x=10, y=2}`
- [x] Тесты JIT: same
- [x] Тесты type error: override non-existing field → error
- [x] GBNF: добавить record-update rule

### B2. Tuple syntax (`(a, b)`)
- [x] Parser: `(expr, expr)` → `ExprKind::Record([fst, snd])`
- [x] Тип: `(a, b)` → `{fst: a, snd: b}` (desugar to record)
- [x] Core IR: desugared at parser level to Record
- [x] Pattern: `(a, b)` в pattern → `Pat::Record([fst, snd])`
- [x] Тесты: `(1, 2).fst == 1`, `(1, 2).snd == 2`
- [x] Тесты: `f (a, b) = a + b; main = f (3, 4)` → 7
- [x] GBNF: добавить tuple rule

### B3. Environment variables
- [x] Eval: builtin `env : String -> String` — `std::env::var().unwrap_or_default()`
- [x] Eval: builtin `env_or : String -> String -> String`
- [x] Тесты: `env "HOME"` → non-empty string
- [x] Тесты: `env "NONEXISTENT_VAR_12345"` → `""`
- [x] Тесты: `env_or "NONEXISTENT" "default"` → `"default"`

### B4. CLI arguments
- [x] Eval: inject `args : [String]` into top-level env
- [x] REPL: parse `--` separator, pass everything after as args
- [x] Тесты: `synoema run file.sno -- a b c` → `args == ["a" "b" "c"]`

### B5. Documentation update (Phase B)
- [x] Обновить `docs/llm/synoema.md` — record update syntax, tuples, env, args
- [x] Обновить `docs/llm/stdlib.md` — env, env_or, args
- [x] Обновить GBNF grammar
- [x] BPE verify `...`

---

## Phase C: Map Type

### C1. Map implementation (в prelude)
- [x] Добавить в prelude: `Map k v = MkMap [(k, v)]`
- [x] Добавить: `empty`, `singleton`, `from_list`
- [x] Добавить: `lookup`, `get`, `has_key`
- [x] Добавить: `insert`, `delete`, `update`
- [x] Добавить: `keys`, `values`, `entries`, `map_values`, `fold_map`, `size`
- [x] Тесты: CRUD operations на Map
- [x] Тесты: `lookup "x" (insert "x" 42 empty) == Ok 42`
- [x] Тесты: `lookup "y" empty == Err "key not found"`
- [x] Тесты: `size (from_list [("a", 1), ("b", 2)]) == 2`
- [x] Тесты: `keys (from_list [("b", 2), ("a", 1)]) == ["a" "b"]` (sorted)

### C2. Documentation update (Phase C)
- [x] Обновить `docs/llm/stdlib.md` — Map API
- [x] Обновить `docs/llm/synoema.md` — Map usage examples

---

## Phase D: JSON Parsing

### D1. JsonValue ADT (в prelude)
- [x] Добавить: `JsonValue = JNull | JBool Bool | JNum Int | JStr String | JArr (List JsonValue) | JObj (List (Pair String JsonValue))`
- [x] Добавить accessors: `json_get`

### D2. json_parse (runtime FFI)
- [x] В `runtime.rs`: ручной recursive descent JSON parser
- [x] Поддержка: null, bool, number, string, array, object
- [x] String escaping: `\"`, `\\`, `\/`, `\n`, `\t`, `\r`
- [x] Возвращает ConNode tree (Ok/Err wrapping JsonValue)
- [x] В `compiler.rs`: зарегистрировать `synoema_json_parse`
- [x] В `eval.rs`: builtin `json_parse`

### D3. json_encode
- [x] В `runtime.rs`: JsonValue tree → JSON string
- [x] Compact format (no pretty-print)
- [x] Тесты: roundtrip

### D4. Тесты + docs
- [x] Тесты: parse simple object, nested, array, null, bool, number
- [x] Тесты: parse error → Err with message
- [x] Тесты: json_get on parsed object
- [x] Обновить `docs/llm/stdlib.md` — JSON API
- [x] Создать example: `examples/json.sno`

---

## Phase E: Code Formatter

### E1. Pretty printer
- [x] Новый файл: `repl/src/fmt.rs` (или отдельный модуль)
- [x] Walk AST → emit formatted text с 2-space indentation
- [x] Handle: function defs, let bindings, conditionals, lists, records, ADTs, modules
- [x] Preserve comments: attach to nearest AST node by span proximity
- [x] Preserve doc comments `---` without reformatting content

### E2. CLI integration
- [x] В `main.rs`: `Some("fmt")` subcommand
- [x] `synoema fmt file.sno` — format in-place
- [x] `synoema fmt dir/` — format all .sno files recursively
- [x] `synoema fmt --check file.sno` — exit 1 if not formatted

### E3. Formatting rules
- [x] 2-space indentation, no tabs
- [x] Trailing whitespace removal
- [x] Max 1 consecutive blank line
- [x] 1 blank line between top-level declarations
- [ ] Operator spacing: `x + y`, `x == y`
- [ ] Import grouping at top, sorted alphabetically

### E4. Тесты
- [x] Idempotency: format(format(code)) == format(code)
- [x] All existing examples/ format correctly
- [x] Comments preserved
- [x] Doc comments preserved
- [x] Тесты: malformed code → error, not crash

---

## Phase F: Build + VS Code

### F1. `synoema build`
- [x] В `main.rs`: `Some("build")` subcommand
- [x] Parse `project.sno` → extract `entry` binding
- [x] Resolve all imports from entry point
- [x] Type check all files (error recovery across files)
- [x] `--jit` flag → JIT compile + run
- [ ] Тесты: build a multi-file project
- [ ] Тесты: missing entry → error "no entry point"

### F2. VS Code extension
- [x] Создать `vscode-extension/package.json`
- [x] Создать `vscode-extension/syntaxes/synoema.tmLanguage.json`
- [x] Создать `vscode-extension/language-configuration.json`
- [x] TextMate: keywords, operators, strings, interpolation, comments, doc comments
- [x] TextMate: type names (capitalized), constructors, function defs
- [x] Bracket matching: `()`, `[]`, `{}`
- [x] Comment toggle: `--`

### F3. Final documentation
- [x] Обновить `CLAUDE.md` — новые команды (init, build, fmt), test count, prelude
- [x] Обновить `context/PROJECT_STATE.md` — полный статус
- [x] Обновить `context/PHASES.md` — новая фаза
- [x] Обновить `--help` output
- [x] Обновить `docs/llm/synoema.md` — все новые конструкции
- [x] Обновить `docs/user/README.md` — полный quickstart с init → write → test → fmt → build
- [x] `docs/mcp.md` — N/A, MCP tools не затронуты

---

## Verify (после всех Phase)
- [x] `cargo test` — 993 tests, 0 failures
- [x] BPE verify для всех новых keywords/operators (`...`, `error`, `env`, `env_or`, `args`)
- [x] GBNF grammar обновлена и покрывает record update + tuple
- [x] Примеры компилируются: 15/16 OK (file_stream.sno — pre-existing relative path issue)
- [x] `synoema init test-project && synoema run test-project/src/main.sno` — работает

### Deferred (nice-to-have, not blocking)
- [ ] Formatter: operator spacing `x + y`, `x == y`
- [ ] Formatter: import grouping/sorting
- [ ] Build: test multi-file project
- [ ] Build: test missing entry error
