---
id: proposal
type: proposal
status: done
---

# Proposal: Doc-as-Code — Self-Documenting Synoema Programs

## Problem

Synoema-приложения не имеют встроенного механизма документирования. Комментарии `--` и `---` семантически идентичны и полностью выбрасываются лексером (`scanner.rs:142-143` → `Token::Newline`). Это создаёт три проблемы:

1. **Нет doc-comments в AST.** Информация о намерении, контрактах и примерах использования теряется при лексировании. Инструменты (MCP-сервер, `synoema doc`, IDE) не могут извлечь документацию из кода
2. **Нет проверяемых примеров (doctests).** Примеры в комментариях — мёртвый текст, который дрифтует при изменении API. Единственная проверяемая документация — тесты в Rust, недоступные пользователю языка
3. **Нет narrative-документации, привязанной к коду.** Tutorials и how-to guides живут в Markdown отдельно от кода. Они не компилируются, не проверяются, могут ссылаться на несуществующие функции

**Влияние на LLM-генерацию:**
- RAG с документацией улучшает качество генерации кода на +24% (CodeRAG-Bench, NAACL 2025)
- Few-shot examples в контексте: +5.7% pass@1 (arxiv 2412.02906)
- Doc-comments рядом с кодом = встроенный RAG для LLM, редактирующей .sno файлы
- MCP tool `docs(fn)` невозможен без doc-comments в AST

## Goal

Три слоя самодокументирования, каждый опциональный:

### Слой 1: Doc-comments (--- → AST)
`---` (три дефиса) → `Token::DocComment(String)` → прикрепляется к следующему `Decl` в AST.

```synoema
--- Sort a list via quicksort. O(n log n) average.
--- example: qsort [3 1 2] == [1 2 3]
--- example: qsort [] == []
qsort : List a -> List a
qsort [] = []
qsort (p:xs) = qsort lo ++ [p] ++ qsort hi
  lo = [x | x <- xs, x <= p]
  hi = [x | x <- xs, x > p]
```

### Слой 2: Doctests (--- example: проверяются)
Строки `--- example: <expr> == <value>` парсятся и проверяются при `synoema test`.
Не влияют на компиляцию/рантайм — работают только в тестовом режиме.

### Слой 3: Guide-файлы (исполняемые .sno с prose)
Любой `.sno` файл с `---` комментариями рендерится как документ через `synoema doc`.
Guide-файлы (`*.guide.sno`) — исполняемые tutorials с `--- guide:` metadata.

```synoema
--- guide: Working with 2D Vectors
--- order: 1
--- requires: basics

--- # Creating vectors
--- Import the Vec2 module:
use Vec2 (make add)

--- Create your first vector:
v1 = make 3 4
--- example: v1.x == 3

--- # Adding vectors
v2 = add v1 (make 1 1)
--- example: v2 == make 4 5

main = v2
```

Guide-файлы компилируются и запускаются как обычные `.sno` → дрифт невозможен.

## Constraints

### Токенная экономика (не нарушается)
- `---` = 1 BPE-токен в cl100k_base (как `--`)
- Doc-comments выбрасываются при desugar (AST → CoreIR) → 0 влияния на JIT/runtime
- LLM не обязана генерировать `---` → overhead при генерации = 0 (opt-in)
- LLM при чтении: prose = ~1.35 tok/word (дешевле кода ~18 tok/line) → docs экономят контекст

### Перформанс (пренебрежимо)
- Compile-time: +String аллокация на каждый `---` (наносекунды, <1% от Cranelift)
- Runtime: 0 (docs не попадают в CoreIR / Cranelift IR / machine code)
- Memory: ~50 bytes × N doc-lines (freed при desugar)
- Doctests: тест-time only, ~2-5s на 100 doctests

### Совместимость
- `--` остаётся обычным комментарием (backwards compatible)
- Файлы без `---` работают идентично текущему поведению
- Existing .sno examples не затрагиваются

## Non-goals

- `Doc` как first-class type (отложено до стабилизации языка, ~50K+ LOC проекты)
- `@source{fn}` / `@signature{fn}` directives (requires Doc type)
- HTML renderer (Phase 19 scope = AST enrichment + doctests; rendering — отдельная задача)
- Structured doc tags (`--- param:`, `--- returns:`) — типы уже документируют это лучше
- Перемещение или реструктуризация существующей документации

## Output

### Изменённые файлы
| Файл | Изменение |
|------|-----------|
| `synoema-lexer/src/scanner.rs` | `---` → `Token::DocComment(String)` |
| `synoema-lexer/src/token.rs` | `DocComment(String)` variant в Token enum |
| `synoema-parser/src/ast.rs` | `doc: Option<Vec<String>>` на Decl |
| `synoema-parser/src/parser.rs` | Collect DocComment tokens, attach to next Decl |
| `synoema-eval/src/eval.rs` | Skip/pass-through doc fields |
| `synoema-core/src/desugar.rs` | Strip doc fields at CoreIR boundary |
| `synoema-repl/src/main.rs` | `synoema test --doctests` subcommand |

### Новые файлы
- Тесты в каждом затронутом crate

### Обновляемая документация (по правилу 7a)
- `docs/llm/synoema.md` — добавить `---` doc-comment syntax
- `docs/specs/language_reference.md` — обновить §2.3 Comments
- `context/PROJECT_STATE.md` — обновить статус
- `context/PHASES.md` — добавить Phase 19
- `CLAUDE.md` — обновить метрики
