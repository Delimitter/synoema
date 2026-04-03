---
id: design
type: design
status: draft
---

# Design: derive(Show, Eq, Ord) для ADT

## Approach: AST-Level Equation Synthesis

### Parser: New Syntax

```
typedef ::= UpperId params "=" variants deriving?
deriving ::= "deriving" "(" UpperId ("," UpperId)* ")"
```

Пример:
```sno
Maybe a = Just a | None
  deriving (Show, Eq)
```

`deriving` парсится как часть TypeDef, хранится в `TypeDef.derives: Vec<String>`.

### AST Change

```rust
// synoema-parser/src/ast.rs
pub struct TypeDef {
    pub name: String,
    pub params: Vec<String>,
    pub variants: Vec<Variant>,
    pub span: Span,
    pub derives: Vec<String>,  // NEW
}
```

### Type Checker: Synthesis

В `infer.rs`, после `register_adt()` (Pass 1), вызвать synthesis:

```rust
// After Pass 1: register ADT constructors
for decl in &program.decls {
    if let Decl::TypeDef(td) = decl {
        if !td.derives.is_empty() {
            let impls = synthesize_derived_impls(
                &td.name, &td.params, &td.variants, &td.derives
            );
            synthetic_impls.extend(impls);
        }
    }
}

// Insert synthetic impls into program for processing
// They go through the same path as manual ImplDecl
```

### derive(Show) — Equation Generation

**Без полей:**
```sno
Color = Red | Green | Blue deriving (Show)

-- Генерирует:
show Red = "Red"
show Green = "Green"
show Blue = "Blue"
```

**С полями:**
```sno
Maybe a = Just a | None deriving (Show)

-- Генерирует:
show (Just _f0) = "Just " ++ show _f0
show None = "None"
```

**С несколькими полями:**
```sno
Pair a b = MkPair a b deriving (Show)

-- Генерирует:
show (MkPair _f0 _f1) = "MkPair " ++ show _f0 ++ " " ++ show _f1
```

**Правила:**
1. Constructor name → string literal
2. Каждое поле → рекурсивный `show` + space separator
3. Если 0 полей: просто строковый литерал имени
4. Если ≥1 полей: `"Name " ++ show f0 ++ " " ++ show f1 ++ ...`

### derive(Eq) — Structural Equality

**Без полей:**
```sno
Color = Red | Green | Blue deriving (Eq)

-- Генерирует:
eq Red Red = true
eq Green Green = true
eq Blue Blue = true
eq _ _ = false
```

**С полями:**
```sno
Maybe a = Just a | None deriving (Eq)

-- Генерирует:
eq (Just _f0) (Just _g0) = eq _f0 _g0
eq None None = true
eq _ _ = false
```

**Правила:**
1. Одинаковые конструкторы → рекурсивный eq по всем полям, `&&` между ними
2. Fallback `eq _ _ = false`
3. Для N полей: `eq f0 g0 && eq f1 g1 && ... && eq fN gN`

### derive(Ord) — Ordering by Declaration Position

**Без полей:**
```sno
Color = Red | Green | Blue deriving (Ord)

-- Red < Green < Blue (по порядку объявления)
-- Генерирует:
cmp Red Red = 0
cmp Red _ = 0 - 1
cmp Green Red = 1
cmp Green Green = 0
cmp Green Blue = 0 - 1
cmp Blue Blue = 0
cmp Blue _ = 1
```

**С полями (lexicographic):**
```sno
Pair a b = MkPair a b deriving (Ord)

-- Генерирует:
cmp (MkPair _f0 _f1) (MkPair _g0 _g1) =
  ? cmp _f0 _g0 != 0 -> cmp _f0 _g0
  : cmp _f1 _g1
```

**Правила:**
1. Вариант с меньшим индексом < вариант с большим индексом
2. Одинаковые варианты → лексикографическое сравнение полей
3. Результат: Int (−1, 0, 1) — не Bool, для composability

### Priority: Manual > Derive

Если пользователь пишет и `derive(Show)`, и `impl Show MyType`:
- Ручной impl побеждает (prepend'ится позже, перекрывает synthetic)
- Это уже работает благодаря механизму equation prepending

### Edge Cases

1. **Пустой ADT** (0 вариантов): не генерировать ничего
2. **Рекурсивный ADT** (`List a = Cons a (List a) | Nil`): рекурсивный show/eq работает благодаря ленивому dispatch
3. **Неизвестный trait в derive**: ошибка типа `unknown_derive_trait`
4. **derive без ADT** (на обычной функции): ошибка парсера

### GBNF Grammar Update

```
typedef = UpperId {" " ident} " = " variants [" deriving " "(" UpperId {"," " " UpperId} ")"]
```
