# Design: Token Savings v2

## Feature 1: Record Punning

### Approach: Parser-level desugar

При парсинге record literal `{...}`, если после `lowerId` стоит `,` или `}` (а не `=`), трактуем как punned field:

```
{x, y, z = x + y}
→ desugar →
{x = x, y = y, z = x + y}
```

### Parser changes (parser.rs, ~L832)

Текущий код:
```rust
let fname = self.expect_lower_id()?;
self.expect(&Token::Assign)?;   // ← requires `=`
let val = self.parse_expr()?;
```

Изменение: после `expect_lower_id()`, проверяем next token:
- Если `=` → parse_expr() как сейчас (explicit field)
- Если `,` или `}` → desugar: val = Expr::Var(fname.clone())

```rust
let fname = self.expect_lower_id()?;
let val = if self.peek() == &Token::Assign {
    self.advance();
    self.parse_expr()?
} else {
    // Punning: {x} → {x = x}
    Expr::new(ExprKind::Var(fname.clone()), span)
};
fields.push((fname, val));
```

### AST impact: None

`ExprKind::Record(Vec<(String, Expr)>)` не меняется — punning полностью десахарится в парсере.

### Pattern punning

Record patterns `{x, y}` → `{x = x, y = y}` аналогично. Текущий код (parser.rs ~L590):
```rust
// Parse record pattern: {field = pat, ...}
```

Такой же подход: если после field name нет `=` → Pat::Var(fname).

---

## Feature 2: Wildcard Import

### Approach: Parser + Module Resolver

#### Parser changes

В `parse_use()` (parser.rs L259), после `(`:
- Если `*` → `UseDecl { names: vec!["*".to_string()], ... }`
- Иначе → парсим список имён как сейчас

```rust
fn parse_use(&mut self) -> PResult<UseDecl> {
    ...
    self.expect(&Token::LParen)?;
    let names = if self.peek() == &Token::Star {
        self.advance();
        vec!["*".into()]
    } else {
        let mut ns = Vec::new();
        while self.peek() != &Token::RParen && self.peek() != &Token::Eof {
            ns.push(self.expect_lower_id()?);
        }
        ns
    };
    self.expect(&Token::RParen)?;
    ...
}
```

#### Token::Star

Проверить: есть ли `Token::Star` в token.rs. `*` используется как `Token::Star` для арифметики. Он уже существует — переиспользуем.

#### Module resolver changes (modules.rs)

В `resolve_modules()`, при обработке `use_decl.names`:
- Если `names == ["*"]` → собрать все имена из модуля
- Иначе → как сейчас

```rust
let names = if use_decl.names == ["*"] {
    // Collect all exported names from module
    module_decls.iter().filter_map(|d| match d {
        Decl::Func { name, .. } if name.starts_with(&format!("{}.", use_decl.module)) => {
            Some(name.strip_prefix(&format!("{}.", use_decl.module))?.to_string())
        }
        _ => None,
    }).collect()
} else {
    use_decl.names.clone()
};
```

#### No changes needed in:
- eval (works on resolved names)
- codegen (works on resolved names)
- type checker (works on resolved decls)

---

## GBNF Grammar Update

```ebnf
use-decl = "use" ws upper-ident ws "(" ws use-names ws ")"
use-names = "*" | lower-ident (ws lower-ident)*
```

## Testing Strategy

| Feature | Test Location | Count |
|---------|--------------|-------|
| Record punning parse | parser tests | 3 |
| Record punning eval | eval tests | 3 |
| Record punning JIT | codegen stress | 2 |
| Pattern punning parse | parser tests | 2 |
| Wildcard import parse | parser tests | 2 |
| Wildcard import eval | eval tests | 3 |
| Wildcard import JIT | codegen stress | 2 |
| Mixed selective+wildcard | parser tests | 1 |
| **Total** | | **18** |
