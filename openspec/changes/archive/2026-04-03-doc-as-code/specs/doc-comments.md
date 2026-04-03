---
id: spec-doc-comments
type: spec
status: done
---

# Spec: Doc-Comments (--- → AST)

## Лексическая спецификация

### Текущее поведение
```
-- comment text\n   → skip_comment() → Token::Newline
--- comment text\n  → skip_comment() → Token::Newline  (идентично --)
```

### Новое поведение
```
--  comment text\n  → skip_comment() → Token::Newline        (без изменений)
--- doc text\n      → scan_doc_comment() → Token::DocComment("doc text")
---- ...            → Token::DocComment("- ...")              (4+ дефисов = doc)
```

Различение: после первых двух `-` проверяем третий символ.
- `peek() == b'-'` → doc-comment (advance, scan text, return DocComment)
- иначе → обычный comment (текущее поведение)

### BPE-alignment
- `---` = 1 BPE-токен в cl100k_base (верифицировано: `---` → token ID 5765)
- `--` = 1 BPE-токен (token ID 313)
- Нейтрально для токенной экономики

### Token variant
```rust
pub enum Token {
    // ... existing variants ...
    DocComment(String),  // NEW: content after "--- "
}
```

`describe()` для DocComment: `"doc comment"`.

## AST-спецификация

### Decl enrichment
```rust
pub enum Decl {
    Func {
        name: String,
        equations: Vec<Equation>,
        span: Span,
        doc: Vec<String>,      // NEW: collected doc-comment lines
    },
    TypeSig(TypeSig),          // doc на следующем Func, не на TypeSig
    TypeDef {
        name: String,
        params: Vec<String>,
        variants: Vec<Variant>,
        span: Span,
        doc: Vec<String>,      // NEW
    },
    TraitDecl { ..., doc: Vec<String> },   // NEW
    ImplDecl { ... },          // без doc (impl — реализация, не API)
}
```

### Module enrichment
```rust
pub struct Module {
    pub name: String,
    pub decls: Vec<Decl>,
    pub span: Span,
    pub doc: Vec<String>,      // NEW: --- before "mod Name"
}
```

### Парсер: правила прикрепления

1. Парсер собирает consecutive `DocComment` токены в `Vec<String>`
2. При встрече `Decl` — прикрепляет собранные doc-comments к этому Decl
3. При встрече `mod` — прикрепляет к Module
4. При встрече кода/expr без Decl — doc-comments отбрасываются (с warning)
5. Пустая строка между `---` и Decl — разрывает привязку (doc отбрасывается)

```synoema
--- This attaches to fac.
fac 0 = 1

--- This is orphaned (no Decl follows immediately).
42
```

## Downstream: что происходит с doc

| Этап | Действие |
|------|----------|
| Lexer | `---` → `Token::DocComment(String)` |
| Parser | Collect → attach to Decl/Module |
| Type checker | Ignore (pass through) |
| Desugar (AST → CoreIR) | **Strip** — doc не попадает в CoreIR |
| Optimizer | N/A (doc уже stripped) |
| Eval (interpreter) | N/A |
| Codegen (JIT) | N/A |
| `synoema doc` | Extract and render |
| MCP `docs()` tool | Extract and return |

## Примеры

### Функция с doc-comment
```synoema
--- Greatest Common Divisor using Euclid's algorithm.
--- example: gcd 12 8 == 4
gcd a 0 = a
gcd a b = gcd b (a % b)
```

### Модуль с doc-comment
```synoema
--- 2D vector arithmetic.
--- Provides construction, addition, dot product.
mod Vec2
  --- Create a vector from x,y coordinates.
  make x y = {x = x, y = y}
```

### Type signature + doc-comment (doc attaches to Func, not TypeSig)
```synoema
--- Sort a list via quicksort.
qsort : List a -> List a
qsort [] = []
qsort (p:xs) = ...
```
