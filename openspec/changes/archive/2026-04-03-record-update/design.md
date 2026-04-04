# Design: Record Update Syntax

## Архитектурное решение

### Ключевой выбор: где реализовывать merge

**Вариант A: Eval-time merge (только interpreter)**
- В `eval.rs`: обработать `ExprKind::RecordUpdate` напрямую
- Pros: просто, не требует знания полей на этапе компиляции
- Cons: в JIT нет доступа к ExprKind (JIT работает с CoreExpr)

**Вариант B: Desugar в Core IR**
- В `desugar.rs`: преобразовать в `let __base = r in {f1 = __base.f1, ..., fi = val_i, ...}`
- Pros: JIT получает стандартный Record + FieldAccess (уже поддерживается)
- Cons: требует знания имён полей при десугаринге — нужен type info

**Вариант C: Гибридный**
- Eval: eval-time merge (проще всего, не требует типов)
- JIT: через desugar, но поля извлекаются runtime через `synoema_record_get` → сложнее

**Решение: Eval-time + JIT через desugar с type info из типизатора**

Для JIT: типизатор уже вычисляет тип base record (с конкретными полями). Можно добавить
`RecordUpdate` в CoreExpr с известным набором полей, известным ПОСЛЕ типизации.

Простейший путь для обоих: добавить `CoreExpr::RecordUpdate` с полным списком полей
(известных из типа base), который JIT компилирует через `record_new/get/set`.

### Финальное решение

```
ExprKind::RecordUpdate { base, updates }
  ↓ eval.rs (interpreter)
    merge base Value::Record with updates
  ↓ desugar.rs (for JIT path)
    CoreExpr::RecordUpdate { base: CoreExpr, all_fields: Vec<String>, updates: Vec<(String, CoreExpr)> }
  ↓ compiler.rs (JIT)
    synoema_record_new(n) + for each field: record_get(base) or compile update val → record_set
```

## Файлы

| Файл | Изменения |
|------|----------|
| `synoema-lexer/src/token.rs` | Добавить `DotDotDot` вариант |
| `synoema-lexer/src/scanner.rs` | Распознавать `...` → `Token::DotDotDot` |
| `synoema-parser/src/ast.rs` | Добавить `RecordUpdate { base, updates }` в `ExprKind` |
| `synoema-parser/src/parser.rs` | Парсить `{ ... expr , field = val ... }` |
| `synoema-types/src/infer.rs` | Вывод типа для RecordUpdate |
| `synoema-core/src/core_ir.rs` | Добавить `CoreExpr::RecordUpdate { base, all_fields, updates }` |
| `synoema-core/src/desugar.rs` | Desugar ExprKind::RecordUpdate → CoreExpr::RecordUpdate |
| `synoema-eval/src/eval.rs` | Eval ExprKind::RecordUpdate |
| `synoema-codegen/src/compiler.rs` | Compile CoreExpr::RecordUpdate |
| `tools/constrained/synoema.gbnf` | Добавить record-update grammar rule |
| `docs/llm/synoema.md` | Добавить `{...r, x = val}` в примеры |

## Детали реализации

### 1. Лексер: `Token::DotDotDot`

```rust
// token.rs
DotDotDot,  // ...

// scanner.rs — в блоке b'.'
b'.' => {
    if self.peek_char() == b'.' {
        self.advance();
        if self.peek_char() == b'.' {
            self.advance();
            Token::DotDotDot
        } else {
            Token::DotDot
        }
    } else {
        Token::Dot
    }
}
```

### 2. AST

```rust
// ast.rs
ExprKind::RecordUpdate {
    base: Box<Expr>,
    updates: Vec<(String, Expr)>,
}
```

### 3. Парсер — в блоке `Token::LBrace`

```rust
Token::LBrace => {
    self.advance();
    self.skip_newlines();
    if self.peek() == &Token::DotDotDot {
        // Record update: {... base, field = val, ...}
        self.advance(); // consume ...
        let base = self.parse_expr()?;
        let mut updates = Vec::new();
        if self.peek() == &Token::Comma {
            self.advance();
            self.skip_newlines();
            while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
                let fname = self.expect_lower_id()?;
                self.expect(&Token::Assign)?;
                let val = self.parse_expr()?;
                updates.push((fname, val));
                self.eat(&Token::Comma);
                self.skip_newlines();
            }
        }
        self.expect(&Token::RBrace)?;
        Ok(Expr::new(ExprKind::RecordUpdate { base: Box::new(base), updates }, span))
    } else {
        // Existing record literal parsing...
    }
}
```

### 4. Типизация

```rust
// infer.rs
ExprKind::RecordUpdate { base, updates } => {
    // Infer base type
    let (s_base, base_ty) = self.infer(env, base)?;
    let base_ty = base_ty.apply(&s_base);

    // Build open record type from updates: {u1: T1, u2: T2 | ρ}
    let mut subst = s_base;
    let mut update_types = Vec::new();
    for (name, expr) in updates {
        let (s, ty) = self.infer(&env.apply(&subst), expr)?;
        subst = s.compose(&subst);
        update_types.push((name.clone(), ty));
    }

    // Create open record with row var to allow "other fields"
    let row_var = self.gen.fresh();
    let open_record = Type::Record(update_types, Some(row_var));

    // Unify base_ty with open_record — ensures update fields exist in base
    let s_unify = unify(&base_ty.apply(&subst), &open_record, &mut self.gen)?;
    let final_subst = subst.compose(&s_unify);

    // Result type = base type (same shape, fields unchanged)
    Ok((final_subst, base_ty.apply(&final_subst)))
}
```

### 5. Core IR

```rust
// core_ir.rs
CoreExpr::RecordUpdate {
    base: Box<CoreExpr>,
    all_fields: Vec<String>,   // все поля из типа base (для JIT)
    updates: Vec<(String, CoreExpr)>,
}
```

### 6. Desugar

```rust
// desugar.rs
ExprKind::RecordUpdate { base, updates } => {
    // Для eval: просто пробросить как CoreExpr::RecordUpdate
    // all_fields будет пустым — eval использует runtime merge
    CoreExpr::RecordUpdate {
        base: Box::new(desugar_expr(fresh, base)),
        all_fields: vec![],  // eval не использует
        updates: updates.iter()
            .map(|(n, e)| (n.clone(), desugar_expr(fresh, e)))
            .collect(),
    }
}
```

### 7. Eval

```rust
// eval.rs
ExprKind::RecordUpdate { base, updates } => {
    let base_val = self.eval(env, base)?;
    let Value::Record(mut fields) = base_val else {
        return Err(err("record update requires a record"));
    };
    for (name, expr) in updates {
        let val = self.eval(env, expr)?;
        if let Some(entry) = fields.iter_mut().find(|(n, _)| n == name) {
            entry.1 = val;
        } else {
            return Err(err(format!("field '{}' not found in record", name)));
        }
    }
    Ok(Value::Record(fields))
}
```

### 8. JIT Compiler

```rust
// compiler.rs
CoreExpr::RecordUpdate { base, all_fields, updates } => {
    // Compile base → base_ptr
    let base_ptr = compile_expr(builder, vars, vc, funcs, module, ctor_tags, base)?;

    // Allocate new record
    let rec_new_id = *funcs.get("synoema_record_new").ok_or_else(|| cerr("missing"))?;
    let len = vc.ins().iconst(types::I64, all_fields.len() as i64);
    let inst = vc.ins().call(rec_new_id, &[len]);
    let rec_ptr = vc.inst_results(inst)[0];

    let rec_set_id = *funcs.get("synoema_record_set").ok_or_else(|| cerr("missing"))?;
    let rec_get_id = *funcs.get("synoema_record_get").ok_or_else(|| cerr("missing"))?;

    // For each field in all_fields: copy from base or compile update
    for (idx, field_name) in all_fields.iter().enumerate() {
        let idx_val = vc.ins().iconst(types::I64, idx as i64);
        let hash = vc.ins().iconst(types::I64, field_hash(field_name) as i64);

        let field_val = if let Some((_, update_expr)) = updates.iter().find(|(n, _)| n == field_name) {
            compile_expr(builder, vars, vc, funcs, module, ctor_tags, update_expr)?
        } else {
            let inst = vc.ins().call(rec_get_id, &[base_ptr, hash]);
            vc.inst_results(inst)[0]
        };

        vc.ins().call(rec_set_id, &[rec_ptr, idx_val, hash, field_val]);
    }

    Ok(rec_ptr)
}
```

**Проблема**: `all_fields` должен быть заполнен из типа base. Потребуется передача type info из типизатора в desugar.

**Упрощение**: Если `all_fields` пуст (eval path), JIT не может работать. Нужно либо:
- Передать тип в desugar — сложно (типы в другом crate)
- Или: в JIT добавить runtime `synoema_record_update(base, updates_ptr)` — проще

**Финальное решение для JIT**: добавить runtime function `synoema_record_update_field(rec, hash, val) -> rec_new` которая копирует рекорд с одним overridden полем. Цепочка для N updates.

Или ещё проще: desugar `{...r, x=e1, y=e2}` → `let __b = r in {x=e1, y=__b.y, z=__b.z, ...}` после типизации, когда поля известны. Это потребует type-annotated AST или post-typecheck pass.

**Для v1**: только interpreter, JIT — будущее. JIT выдаёт fallback/error для RecordUpdate.
