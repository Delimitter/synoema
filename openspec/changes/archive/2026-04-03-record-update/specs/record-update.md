# Spec: Record Update Syntax

## Syntax

```sno
updated = {...base, field1 = val1, field2 = val2}
```

Создаёт новый record: копирует все поля из `base`, перезаписывает указанные.

## Примеры

```sno
p = {x = 1, y = 2, z = 3}
q = {...p, x = 10}           -- {x = 10, y = 2, z = 3}
r = {...p, x = 10, y = 20}   -- {x = 10, y = 20, z = 3}

config = {host = "localhost", port = 8080}
prod = {...config, host = "prod.example.com"}
```

## Лексер

| Токен | Текст | BPE (cl100k_base) |
|-------|-------|-------------------|
| `Token::DotDotDot` | `...` | 1 токен ✓ |

Изменение в `scanner.rs`:
```rust
b'.' => match (self.match_char(b'.'), ...) {
    (true, ...) if next == b'.' => { self.advance(); Token::DotDotDot }
    (true, _)                   => Token::DotDot
    (false, _)                  => Token::Dot
}
```

## AST

```rust
// parser/src/ast.rs
ExprKind::RecordUpdate {
    base: Box<Expr>,
    updates: Vec<(String, Expr)>,
}
```

## Парсер

Внутри `{` — если следующий токен `...`:
```
'{' '...' expr ',' (field '=' expr (',' field '=' expr)*)? '}'
```

## Типизация

Вывод типа через row-polymorphism:

```
Γ ⊢ base : {f1: T1, ..., fN: TN}   (closed record, row = None after resolution)
Γ ⊢ val_i : T_i    ∀ i ∈ updates   (тип должен совпадать с existing field)
────────────────────────────────────────────────────────────────────────────────
Γ ⊢ {...base, fi = val_i, ...} : {f1: T1, ..., fN: TN}
```

Стратегия: unify base с open record `{updates | ρ}`, результат = тот же тип что у base.

Ошибка: override поля, которого нет в base → type error.

## Core IR / Desugar

`RecordUpdate` не добавляется в CoreExpr — десугарируется в parser или desugar.rs:

```
{...r, x = e1, y = e2}
→ (run-time desugar in eval, or type-driven compile-time if fields known)
```

Eval-time подход (проще, достаточно для interpreter):
- Вычислить base → `Value::Record(base_fields)`
- Вычислить каждый `val_i`
- Merge: скопировать все `base_fields`, заменить совпадающие ключи

JIT-подход:
- Скомпилировать base → rec_ptr
- Вызвать `synoema_record_new(len)` для нового рекорда
- Для каждого поля (известно из base type): `synoema_record_get` + copy, затем override updates
- Либо: desugar в Core IR как `let __base = r in {f1=__base.f1, ..., fi=val_i, ...}`

## Eval

```rust
ExprKind::RecordUpdate { base, updates } => {
    let base_val = self.eval(env, base)?;
    let Value::Record(mut fields) = base_val else {
        return Err(err("record update requires a record base"));
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

## JIT

Вариант A — desugar в Core IR (рекомендуется):

```
let __base = r in {f1 = __base.f1, f2 = __base.f2, ..., fi = val_i, ...}
```

Преимущество: JIT уже умеет компилировать `Record` и `FieldAccess`. Никаких новых CoreExpr.

Вариант B — новый runtime call `synoema_record_copy` — излишне сложен.

**Решение**: Вариант A. Desugar в `desugar.rs`, если имена полей известны из типа. Если тип неизвестен — fallback в eval-time.

## Тесты

### Interpreter
```sno
-- Basic update
{...{x=1, y=2}, x=10} == {x=10, y=2}

-- Multiple field update
{...{x=1, y=2, z=3}, x=10, y=20} == {x=10, y=20, z=3}

-- Nested base expression
let r = {a=1, b=2}
{...r, a=99} == {a=99, b=2}

-- Type error: non-existent field
-- {..{x=1}, nonexistent=2} → type error
```

### JIT
Same as interpreter — должны выдавать идентичные результаты.

## GBNF

Добавить в `tools/constrained/synoema.gbnf`:
```
record-update ::= "{" "..." expr ("," field-assign)+ "}"
```

## Документация

Обновить `docs/llm/synoema.md` — добавить в секцию Records:
```
{...r, x = val}   -- record update: copy r, override x
```
