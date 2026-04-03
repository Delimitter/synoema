---
id: tasks
type: tasks
status: done
---

# Tasks: Doc-as-Code (Phase 19)

## Слой 1: Doc-comments (--- → AST)

- [x] **T1: Lexer — DocComment token**
  - `synoema-lexer/src/token.rs`: добавить `DocComment(String)` в Token enum
  - `token.rs`: `describe()` → `"doc comment"`
  - `synoema-lexer/src/scanner.rs`: при `--` + `peek() == b'-'` → advance, scan text до EOL, return `Token::DocComment(text.trim())`
  - Тесты: `--- doc text` → DocComment("doc text"), `-- regular` → Newline (unchanged), `---- many dashes` → DocComment("- many dashes")

- [x] **T2: Parser — collect and attach doc-comments**
  - `synoema-parser/src/ast.rs`: добавить `doc: Vec<String>` к Func, TypeDef, TraitDecl, Module
  - `synoema-parser/src/parser.rs`: перед parse_decl — собирать consecutive DocComment tokens в Vec<String>
  - При встрече Decl → прикрепить. При встрече не-Decl → отбросить (TODO: warning)
  - Тесты: `--- doc\nfac 0 = 1` → Func { doc: ["doc"], .. }, multiple `---` lines → multiple entries, orphaned `---` → empty doc

- [x] **T3: Desugar — strip doc at CoreIR boundary**
  - `synoema-core/src/desugar.rs`: убедиться что doc не попадает в CoreIR (doc — поле AST Decl, CoreIR не имеет doc → автоматически stripped)
  - Проверить: CoreExpr не содержит doc → OK, ничего менять не надо (verify only)

- [x] **T4: Downstream crates — adapt to new AST**
  - `synoema-types/src/infer.rs`: ignore doc field (pattern match update)
  - `synoema-eval/src/eval.rs`: ignore doc field (pattern match update)
  - `synoema-codegen/src/compiler.rs`: CoreIR не содержит doc → no changes expected
  - `synoema-diagnostic`: no impact
  - `cargo test` — все 689+ тестов проходят

- [x] **T5: Update existing examples with doc-comments**
  - Добавить `---` doc-comments к 3-4 examples (quicksort, geometry, modules, higher_order)
  - Показать pattern: `--- description\n--- example: expr == val\nfunc args = body`

## Слой 2: Doctests

- [x] **T6: Doctest extractor**
  - Новый модуль в `synoema-repl` или `synoema-eval`: `doctest.rs`
  - Input: `Program` (AST с doc-comments)
  - Output: `Vec<Doctest>` where `Doctest { span, expr_str, expected: Option<String> }`
  - Парсинг: для каждого Decl → filter doc lines с prefix `example:` → split по `==` если есть

- [x] **T7: Doctest runner**
  - Input: `Vec<Doctest>` + eval environment (loaded module)
  - Для каждого doctest: parse expr → typecheck → eval
  - Если `expected` есть: eval expected → compare с результатом
  - Если `expected` нет: assert no runtime error
  - Output: pass/fail per doctest + summary

- [x] **T8: CLI integration — `synoema test`**
  - `synoema-repl/src/main.rs`: добавить subcommand `test <file-or-dir>`
  - Для файла: load → extract doctests → run → report
  - Для директории: рекурсивно найти все .sno → test каждый
  - Exit code: 0 если все pass, 1 если есть failures
  - Тесты: doctest pass, doctest fail, no doctests (OK), syntax error in doctest

## Слой 3: Guide-файлы (базовый)

- [x] **T9: Guide metadata parser**
  - Из top-level doc-comments файла: extract `guide:`, `order:`, `requires:`
  - Struct: `GuideMeta { title: Option<String>, order: f64, requires: Vec<String> }`
  - Если нет `guide:` → None (обычный .sno)

- [x] **T10: `synoema doc --format md` — basic Markdown output**
  - Input: .sno file
  - Output: Markdown where `---` lines → prose, code → fenced code blocks, `--- example:` → code + expected
  - Type signatures auto-extracted (infer types, show in docs)
  - Для directory: generate index.md с listing

## Документация и финализация

- [x] **T11: Обновить документацию (правило 7a)**
  - `docs/llm/synoema.md`: добавить `---` doc-comment в таблицу
  - `docs/specs/language_reference.md`: обновить §2.3 Comments (doc vs regular)
  - `context/PROJECT_STATE.md`: Phase 19, обновить test count
  - `context/PHASES.md`: добавить Phase 19
  - `CLAUDE.md`: обновить метрики (тесты, фичи)
  - `docs/user/README.md`: упомянуть `synoema test` и `synoema doc`

- [x] **T12: BPE verification**
  - Проверить `---` в `tools/bpe-verify/verify_bpe.py` (должен быть 1 token)
  - Добавить `---` в список проверяемых операторов если отсутствует
  - Обновить GBNF grammar: `tools/constrained/synoema.gbnf` — добавить doc-comment rule
