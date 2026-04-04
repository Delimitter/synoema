# Spec: Record Update Syntax (Spread)

## Syntax

```sno
updated = {...original, field1 = val1, field2 = val2}
```

Семантика: создать новый record, копируя все поля из `original`, перезаписывая указанные.

## Примеры

```sno
p = {x = 1, y = 2, z = 3}
q = {...p, x = 10}           -- {x = 10, y = 2, z = 3}
r = {...p, x = 10, y = 20}   -- {x = 10, y = 20, z = 3}

-- вложенный:
config = {host = "localhost", port = 8080}
prod = {...config, host = "prod.example.com"}
```

## Pipeline

| Crate | Изменение |
|-------|----------|
| Lexer | `Token::DotDotDot` (`...`) — 1 BPE-токен ✓ |
| Parser | `ExprKind::RecordUpdate { base: Box<Expr>, fields: Vec<(String, Expr)> }` |
| Types | Infer base as record row, unify override fields |
| Core IR | Desugar → field extraction + new record construction |
| Eval | `eval_record_update(base_val, overrides)` |
| JIT | Allocate new RecordNode, copy fields, apply overrides |

## BPE Verification

`...` = 1 BPE-токен на cl100k_base ✓

## Type Checking

```
Γ ⊢ base : {field1: T1, ..., fieldN: TN | ρ}
Γ ⊢ val_i : T_i   (для каждого переопределённого поля)
────────────────────────────────────────────────────
Γ ⊢ {...base, field_i = val_i} : {field1: T1, ..., fieldN: TN | ρ}
```

Ошибка: если override field не существует в base record (strict, no extension).

## Что НЕ входит

- Nested update (`{...r, a.b = 5}`) — слишком сложно для alpha
- Delete field (`{...r, -field}`)
- Merge двух records (`{...r1, ...r2}`)
