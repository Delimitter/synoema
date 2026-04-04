# Design: LLM Production Readiness

## Dependency Graph (порядок реализации)

```
                    ┌─────────────┐
                    │  synoema    │
                    │    init     │
                    └──────┬──────┘
                           │ (independent)
              ┌────────────┼────────────────┐
              ▼            ▼                ▼
      ┌──────────┐  ┌───────────┐   ┌──────────┐
      │  Result  │  │  Record   │   │  VS Code │
      │   type   │  │  Update   │   │   ext    │
      │ +prelude │  │  {...r}   │   └──────────┘
      └────┬─────┘  └───────────┘
           │
           ▼
      ┌──────────┐
      │   Map    │ (requires Result for lookup)
      │   type   │
      └────┬─────┘
           │
           ▼
      ┌──────────┐
      │   JSON   │ (requires Result + Map)
      │  parsing │
      └──────────┘

      ┌──────────┐  ┌──────────┐  ┌──────────┐
      │ env/args │  │ formatter│  │  build   │
      └──────────┘  └──────────┘  └──────────┘
          (independent)   (independent)   (independent)
```

## Phased Delivery

### Phase A: Project Init + Prelude + Result (foundation)
Всё остальное зависит от этого. `synoema init` создаёт проект, prelude загружает Result.

### Phase B: Record Update + Env/Args (independent, small)
Маленькие, не зависят друг от друга. Можно параллельно.

### Phase C: Map type (depends on Result)
Association list в prelude. Чистые функции.

### Phase D: JSON parsing (depends on Result + Map)
Runtime FFI для json_parse, prelude для JsonValue ADT.

### Phase E: Formatter (independent, large)
AST-based pretty printer. Самостоятельная задача.

### Phase F: Build command + VS Code (finishing touches)
Build — тонкая обёртка над import resolution. VS Code — TextMate grammar.

---

## Design Decisions

### 1. Prelude mechanism

**Подход:** `include_str!("prelude.sno")` в Rust — prelude встроен в бинарник.

```rust
// eval.rs / compiler.rs
const PRELUDE: &str = include_str!("../../prelude/prelude.sno");

fn load_prelude(env: &mut Env) {
    let ast = parse(PRELUDE);
    let typed = typecheck(ast);
    eval_program(typed, env);
}
```

Вызывается перед eval пользовательской программы. JIT: prelude компилируется один раз.

**Расположение файла:** `lang/prelude/prelude.sno`

**Важно:** prelude НЕ показывается как import — это прозрачная часть runtime.

### 2. Result — ADT в prelude, не hardcoded

Result определяется как обычный ADT в prelude.sno. Комбинаторы — обычные функции. Никаких изменений в компиляторе.

### 3. Map — sorted assoc list в prelude

Чистые функции в prelude.sno. Единственное ограничение: O(n) lookup. Для LLM-приложений (десятки ключей) — достаточно.

Если перформанс станет проблемой — потом можно добавить hash-based Map в runtime.rs как FFI, с тем же API.

### 4. Record update — новый ExprKind

```rust
ExprKind::RecordUpdate {
    base: Box<Expr>,
    updates: Vec<(String, Expr)>,
}
```

Desugar → extracting all fields from base, creating new record with overrides:
```
{...r, x = 42}
→
let __base = r in {x = 42, y = __base.y, z = __base.z}
```

Тип base record должен быть known (конкретный row, не open). Ошибка если поле в updates не существует в base.

### 5. JSON parser — ручной recursive descent в Rust

~150 строк в runtime.rs. Возвращает Synoema values (ConNode для ADT constructors). Не serde_json.

```rust
pub extern "C" fn synoema_json_parse(s: i64) -> i64 {
    // s = tagged string ptr
    // returns: tagged ConNode (Ok/Err containing JsonValue tree)
}
```

### 6. Formatter — AST-based

Lexer → Parser → AST → PrettyPrint. Comments attached to AST nodes by span proximity.

Новый модуль в synoema-repl (или отдельный crate synoema-fmt — решить по объёму).

### 7. `synoema init` — в main.rs

Новая CLI subcommand. Чистый Rust: `std::fs::create_dir_all`, `std::fs::write`. Шаблоны — `include_str!` из `templates/` директории.

### 8. `synoema build` — тонкая обёртка

Парсит `project.sno` → находит `entry` → вызывает существующий `run_file` / `jit_file` с import resolution.

### 9. Tuple syntax

`(a, b)` → sugar для `{fst = a, snd = b}`. Парсер: если `(` followed by expr `,` → parse as tuple → desugar to record.

Нужен новый `ExprKind::Tuple(Vec<Expr>)` + desugar в `{fst = ..., snd = ...}` (для 2-tuple) / `{_0 = ..., _1 = ..., _2 = ...}` (для N-tuple).

**BPE:** `,` внутри `()` = tuple. `,` внутри `{}` = record. `,` внутри `[]` = error (lists are space-separated).

### 10. VS Code extension

TextMate JSON grammar. Собирается через `vsce package`. Публикация на marketplace — вне scope (alpha).

---

## Files Changed (summary)

| File | Changes |
|------|---------|
| `repl/src/main.rs` | +`init` subcommand, +`build` subcommand, +`fmt` subcommand |
| `lang/prelude/prelude.sno` | NEW: Result, Map, combinators, JsonValue |
| `lang/templates/` | NEW: init templates (CLAUDE.md, main.sno, etc.) |
| `lexer/src/lexer.rs` | +`Token::DotDotDot` |
| `parser/src/parser.rs` | +`ExprKind::RecordUpdate`, +`ExprKind::Tuple` |
| `types/src/infer.rs` | Infer RecordUpdate, Tuple |
| `core/src/desugar.rs` | Desugar RecordUpdate → field extraction, Tuple → Record |
| `eval/src/eval.rs` | +prelude loading, +`env`/`env_or` builtins, +`args`, +`error` |
| `codegen/src/compiler.rs` | +RecordUpdate in JIT |
| `codegen/src/runtime.rs` | +`synoema_json_parse`, +`synoema_json_encode` |
| `tools/constrained/synoema.gbnf` | +record-update, +tuple |
| `vscode-extension/` | NEW: TextMate grammar + package.json |
