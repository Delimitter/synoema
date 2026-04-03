# Design: Linear Types Implementation

## Architecture Decisions

### D1: Multiplicity на уровне Arrow, не переменных

**Решение:** `Type::Arrow` получает поле `Multiplicity`.

**Альтернативы:**
- Annotate bindings in TypeEnv — проще, но не выражает линейность в higher-order типах
- Kind-level linearity — слишком инвазивно

**Обоснование:** Позволяет записать `Int -o Int` как тип (compositional). Minimal change to `Subst` — multiplicity переменные входят в тот же `TyVarId` namespace.

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Multiplicity {
    One,          // linear: must use exactly once
    Many,         // unrestricted: can use 0..n times (current default)
    Var(TyVarId), // multiplicity variable (for inference)
}

// Type::Arrow changes from:
//   Arrow(Box<Type>, Box<Type>)
// to:
//   Arrow(Box<Type>, Multiplicity, Box<Type>)
```

### D2: Синтаксис `-o` для линейной стрелки

**Решение:** Новый токен `LinearArrow` в лексере.

**BPE проверка:** `-o` в cl100k_base — 1 токен (`-o` = token id 12). ✓

**В парсере:** При виде `-o` создаётся `TypeExprKind::Arrow { linear: true }`.

**В AST:** `TypeExprKind::Arrow` получает флаг `linear: bool` (не `Multiplicity`, чтобы не усложнять AST).

### D3: Usage Tracking — отдельная структура, не часть TypeEnv

**Решение:** `LinearContext` — HashMap из `Name → UsageCount`, живёт рядом с TypeEnv в `Infer`.

```rust
#[derive(Debug, Clone, PartialEq)]
enum UsageCount { Unused, Once, Many }

struct LinearContext {
    usages: HashMap<String, UsageCount>,
    linear_vars: HashSet<String>,
}
```

**Почему отдельно от TypeEnv:** TypeEnv — персистентный, shared, используется для lookup. LinearContext — мутабельный, per-scope. Их нельзя смешивать без больших рефакторингов.

**Альтернатива:** Мутировать TypeEnv в place — нарушает immutable-first дизайн, сложнее в case expressions.

### D4: Opt-in совместимость — НЕ ломаем существующий код

**Решение:** Usage tracking активируется ТОЛЬКО для переменных, binding которых введён с linear-typed функцией.

Все существующие `->` arrows → `Multiplicity::Many`. Поведение не меняется.

### D5: Case expressions — intersection semantics

В каждой ветви case linear переменные из enclosing scope должны использоваться одинаково. После анализа всех веток — объединяем usage counts:

```rust
// Pseudocode
for branch in branches {
    let branch_usages = infer_in_branch(branch);
    merge_usages(&mut result, branch_usages);
    // если ветви не согласованы — error
}
```

**Реализация:** Сохраняем копию LinearContext перед каждой веткой, сравниваем после.

### D6: Новые error codes

```rust
// в synoema-diagnostic/src/lib.rs (codes module):
pub const LINEAR_UNUSED: &str = "linear_unused";
pub const LINEAR_DUPLICATE: &str = "linear_duplicate";
```

Новые варианты в `TypeErrorKind`:
```rust
LinearUnused { name: String },
LinearDuplicate { name: String },
```

### D7: Interpreter — runtime assertion, не static check

В `synoema-eval`: linеарность уже проверена type checker'ом. Дополнительного runtime overhead нет. Линейные переменные bind/use как обычные.

## Изменённые файлы

| Файл | Изменение |
|------|-----------|
| `synoema-lexer/src/scanner.rs` | Токен `LinearArrow` (`-o`) |
| `synoema-parser/src/parser.rs` | `TypeExprKind::Arrow { linear }`, парсинг `-o` |
| `synoema-types/src/types.rs` | `Multiplicity` enum, `Arrow(_, Mult, _)` |
| `synoema-types/src/unify.rs` | Unification для `Multiplicity` |
| `synoema-types/src/infer.rs` | `LinearContext`, usage tracking в `infer_inner` |
| `synoema-types/src/error.rs` | `LinearUnused`, `LinearDuplicate` variants |
| `synoema-diagnostic/src/lib.rs` | Новые codes |
| `synoema-types/src/tests.rs` | Новые тесты |

## Не меняется

- `synoema-core/` — Core IR без изменений (linearity erased after typecheck)
- `synoema-eval/` — нет изменений (type-safe по результату typecheck)
- `synoema-codegen/` — нет изменений (Phase A)
- `synoema-repl/` — нет изменений

## Ключевой риск

**Сложность инференции:** Algorithm W не отслеживает usage нативно. Добавление `LinearContext` в `infer_inner` требует осторожности — особенно в App (правило контекстного split: `Γ₁, Γ₂`). В Synoema нет контекстного split, вместо этого мы просто count usages и проверяем по выходу из lambda/let scope.

**Решение:** Simplified approach — не делаем полный context split (слишком сложно). Вместо этого: count usage per variable, проверяем инвариант при выходе из scope.
