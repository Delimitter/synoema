# Tasks: Linear Types (Phase A)

## Checklist

- [ ] T1: Добавить токен `LinearArrow` (`-o`) в лексер
- [ ] T2: Добавить `Multiplicity` enum и обновить `Type::Arrow` в types.rs
- [ ] T3: Обновить `Subst`, `Type::ftv`, `Type::apply`, `Display` для `Multiplicity`
- [ ] T4: Обновить `unify` для Arrow с Multiplicity
- [ ] T5: Добавить `LinearUnused` / `LinearDuplicate` в `TypeErrorKind` и diagnostic codes
- [ ] T6: Добавить парсинг `-o` в `TypeExprKind::Arrow`
- [ ] T7: Реализовать `LinearContext` + usage tracking в `infer.rs`
- [ ] T8: Написать тесты (≥10) для linear type checking
- [ ] T9: Убедиться, что все существующие тесты проходят (0 failures, 0 warnings)

## Детализация задач

### T1: Лексер — токен LinearArrow
Файл: `lang/crates/synoema-lexer/src/scanner.rs`
- Добавить `Token::LinearArrow` после существующих Arrow-подобных токенов
- Добавить сканирование `-o` (дефис + 'o' не как идентификатор)
- ВАЖНО: `-o` должен сканироваться раньше, чем просто `-` (MinusOp)

### T2: Type::Arrow + Multiplicity
Файл: `lang/crates/synoema-types/src/types.rs`
- Добавить `pub enum Multiplicity { One, Many, Var(TyVarId) }`
- Изменить `Arrow(Box<Type>, Box<Type>)` → `Arrow(Box<Type>, Multiplicity, Box<Type>)`
- Обновить конструктор `Type::arrow(a, b)` → `Type::arrow_un(a, b)` (Many) + `Type::arrow_lin(a, b)` (One)
- Добавить `TyVarGen::fresh_mult()` для multiplicity variables

### T3: Subst / ftv / apply / Display
Файл: `lang/crates/synoema-types/src/types.rs`
- `Type::ftv()`: собирать `Multiplicity::Var(id)` в HashSet
- `Type::apply(subst)`: применять subst к Multiplicity::Var
- `Subst`: multiplicity переменные используют тот же `TyVarId` namespace (избегаем отдельного HashMap)
  - Хранить `Multiplicity::Var` в `Subst` как `Type::Con("__mult_one")` / `Type::Con("__mult_many")`? — НЕТ, слишком грязно
  - Добавить отдельный `HashMap<TyVarId, Multiplicity>` в `Subst`
- `Display`: `->` для Many, `-o` для One

### T4: Unification для Multiplicity
Файл: `lang/crates/synoema-types/src/unify.rs`
- Arrow case: унифицировать обе стороны + multiplicities
- `unify_mult(m1, m2) -> Result<Subst_Mult, TypeError>`
  - `Many == Many` → ok
  - `One == One` → ok
  - `Var(a) == m` → bind mult var
  - `Many == One` → TypeError (нельзя передать linear туда, где ожидается unrestricted)

### T5: Error kinds + diagnostic codes
Файлы: `synoema-types/src/error.rs`, `synoema-diagnostic/src/lib.rs`
- Добавить в `TypeErrorKind`:
  ```rust
  LinearUnused { name: String },
  LinearDuplicate { name: String },
  ```
- Добавить в `codes`:
  ```rust
  pub const LINEAR_UNUSED: &str = "linear_unused";
  pub const LINEAR_DUPLICATE: &str = "linear_duplicate";
  ```
- Обновить `Display` для новых вариантов
- Обновить `type_err()` в `synoema-eval/src/lib.rs`

### T6: Парсинг `-o`
Файл: `lang/crates/synoema-parser/src/parser.rs`
- Обновить `TypeExprKind::Arrow` (или добавить `linear: bool` поле в существующий вариант)
- В `parse_type()`: при виде `LinearArrow` создавать `TypeExprKind::LinearArrow`
- Pratt-уровень для `-o` = тот же, что для `->` (правая ассоциативность)

### T7: LinearContext + usage tracking
Файл: `lang/crates/synoema-types/src/infer.rs`
- Добавить `LinearContext` struct (см. design.md)
- В `Infer` добавить поле `linear_ctx: LinearContext`
- В `infer_inner → ExprKind::Var`: если переменная в `linear_vars`, increment usage
- В `infer_inner → ExprKind::Lam`: при вводе linear-аргумента — добавить в linear_ctx; при выходе — проверить usage
- В `infer_inner → ExprKind::Let/LetRec`: аналогично
- В `infer_inner → ExprKind::Case`: intersection semantics (копировать ctx перед каждой веткой)

### T8: Тесты
Файл: `lang/crates/synoema-types/src/tests.rs`
- `linear_arrow_type_correct` — `f : Int -o Int` проходит type check
- `linear_unused_error` — переменная linear, не используется → error
- `linear_duplicate_error` — переменная linear, используется дважды → error
- `unrestricted_unchanged` — существующий код без -o работает
- `linear_in_case_ok` — linear var используется в обеих ветках → ok
- `linear_in_case_err` — linear var используется в одной ветке → error
- `linear_nested` — вложенные линейные функции
- `linear_hof` — передача linear функции как аргумент
- `linear_typecheck_display` — Display `-o` корректно
- `linear_infer_arrow` — инференция multiplicity

### T9: Полный прогон тестов
```bash
cd lang && cargo test
```
0 failures, 0 warnings.
