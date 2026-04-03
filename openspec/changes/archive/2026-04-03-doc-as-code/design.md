---
id: design
type: design
status: done
---

# Design: Doc-as-Code

## Ключевые решения

### D1: `---` (три дефиса) как doc-comment маркер

**Решение:** `---` → DocComment, `--` → обычный comment (stripped).

**Альтернативы:**
- `--|` (Haddock-style) — отклонено: не 1 BPE-токен, ломает BPE-alignment правило
- `{- -}` block comments — отклонено: требует терминал закрытия, нарушает минимализм
- `///` (Rust-style) — отклонено: `/` уже оператор деления, двусмысленность в лексере

**Обоснование:** `---` уже распознаётся лексером (3-й дефис после `--`). Различение: `peek() == b'-'` после второго дефиса. BPE-нейтрально (1 токен). Backwards compatible — `--` не затронут.

### D2: Doc-comments прикрепляются к Decl, не к Expr

**Решение:** doc-comments attach только к Decl (Func, TypeDef, TraitDecl) и Module.

**Обоснование:**
- Документируются API-поверхности (функции, типы, модули), не выражения
- Go, Rust, Haskell, Elixir — все привязывают docs к declarations
- Orphaned doc-comments (без Decl) → warning при parse, не ошибка

### D3: Vec<String> вместо structured doc

**Решение:** `doc: Vec<String>` — плоский список строк.

**Альтернативы:**
- `doc: Option<DocBlock>` с `DocBlock { summary, examples, params }` — отклонено: over-engineering для alpha
- `doc: Option<String>` (одна строка) — отклонено: многострочные docs нужны

**Обоснование:** Vec<String> — минимальная структура. Парсинг `example:` prefix → извлечение doctests — на стороне потребителя (test runner, doc generator), не в AST. Позволяет расширять формат без изменения AST.

### D4: Doctests через interpreter, не JIT

**Решение:** doctests выполняются через `synoema-eval` (tree-walking interpreter).

**Обоснование:**
- Interpreter = reference implementation (правило 3: interpreter-first)
- Interpreter проще для отладки doctest failures
- JIT-doctests — возможно в будущем через флаг `--jit`
- Нет overhead JIT-компиляции для маленьких выражений

### D5: Doctests в контексте всего модуля

**Решение:** doctest выполняется после полной загрузки модуля.

**Альтернативы:**
- Isolated execution (каждый doctest = отдельный module load) — отклонено: Rust делает это и это причина тормозов (`O(N)` compilations вместо `O(1)`)
- Sequential (doctest видит только предыдущие definitions) — отклонено: нарушает Synoema semantics (все определения взаимно-рекурсивны)

**Обоснование:** Synoema загружает все определения перед eval (mutual recursion). Doctest запускается в том же environment → видит все функции модуля. Изоляция бесплатная: immutability by design → нет shared mutable state.

### D6: Guide metadata в doc-comments, не в отдельном файле

**Решение:** `--- guide: Title` в начале .sno файла.

**Альтернативы:**
- YAML frontmatter (`---\ntitle: ...\n---`) — отклонено: конфликт с `---` doc-comment syntax
- Отдельный `guide.toml` — отклонено: нарушает принцип "один файл = всё"
- Pragma в коде (`#guide "Title"`) — отклонено: новый синтаксис без оправдания

**Обоснование:** Metadata — это doc-comments с known prefixes. Парсится тем же механизмом. Не требует нового синтаксиса. Guide metadata опционален — без него файл = обычный .sno.

### D7: `synoema test` вместо `synoema doctest`

**Решение:** единая команда `synoema test file.sno` запускает doctests.

**Обоснование:** не плодить команды. Пользователь пишет `synoema test` — получает все проверки. В будущем: `synoema test` может запускать и property-based тесты, assertions, etc.

### D8: Scope Phase 19 — AST + doctests, без HTML rendering

**Решение:** Phase 19 = lexer/parser/AST enrichment + doctest runner + `synoema doc --format md`.

**Обоснование:**
- HTML rendering — большая задача (CSS, navigation, search)
- Markdown output достаточен для первой версии
- MCP tool `docs()` — отдельный change (зависит от MCP server infrastructure)
- Incremental delivery: каждая часть ценна сама по себе

### D9: Влияние на большие проекты (прогноз)

**До ~20K LOC:** Doc-comments + doctests + guide-files достаточно.
**20K-50K LOC:** Нужен search по docs (MCP tool, `synoema doc --search`).
**50K+ LOC:** Рассмотреть Doc type (Unison-style) для type-checked cross-references.

Phase 19 закладывает фундамент (doc в AST), на котором строятся все три уровня.
