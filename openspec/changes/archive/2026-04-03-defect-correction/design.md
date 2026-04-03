# Design: Defect Correction

## D1: Row Polymorphism Fix

**File:** `unify.rs`, ветка `(Some(r1), Some(r2))` при `r1 == r2`

**Approach:** При r1 == r2 — extra_in_1 и extra_in_2 должны быть пустыми (оба типа имеют одинаковый хвост, значит named fields должны совпадать — общие поля уже унифицированы выше). Если непустые — Mismatch error.

**Rationale:** Два record типа с одной row-переменной `{a: T1, b: T2 | r}` и `{a: T3, c: T4 | r}` — несовместимы, потому что `r` обозначает одно и то же расширение. Поле `b` есть в первом но не во втором, а `c` — наоборот. Это противоречие.

## D2: RuntimeError в CoreExpr

**Files:** `core_ir.rs`, `desugar.rs`, `eval.rs`, `compiler.rs`, `runtime.rs`

**Approach:**
1. Добавить `RuntimeError(String)` в `CoreExpr`
2. В `desugar.rs:164` — заменить `Lit(Int(0))` на `RuntimeError("non-exhaustive patterns".into())`
3. Interpreter: `RuntimeError(msg)` → panic с msg
4. JIT: `RuntimeError(msg)` → вызов FFI `synoema_match_error(ptr, len)` → eprintln + process::exit(1)

## D3: Constructor Pattern Type Lookup

**File:** `infer.rs`, `infer_pattern`

**Approach:** Изменить сигнатуру `infer_pattern` чтобы принимать `&TypeEnv`. Для `Pat::Con(name, sub_pats)`:
1. Lookup name в env
2. Instantiate scheme
3. Unroll тип: `T1 -> T2 -> ... -> R` → args=[T1,T2,...], result=R
4. Проверить len(sub_pats) == len(args)
5. Unify каждый sub_pat type с соответствующим arg type
6. Вернуть result type R

**Fallback:** Если конструктор не найден в env — оставляем fresh type variable (для обратной совместимости с ADT, которые декларируются в том же модуле).
