# Spec: Type System Soundness Fixes

## S1: Row Polymorphism Unification (CRITICAL)

**Location:** `synoema-types/src/unify.rs`, ветка `(Some(r1), Some(r2))` при `r1 == r2`

**Current behavior:** Когда `r1 == r2`, `extra_in_1` и `extra_in_2` игнорируются — поля не унифицируются.

**Required behavior:** Если поле с одинаковым именем присутствует в `extra_in_1` и `extra_in_2`, их типы ДОЛЖНЫ быть унифицированы. Поля, уникальные для одной стороны — ошибка (при одном row variable нельзя иметь разные наборы).

**Note:** При `r1 == r2` extra_in_1 и extra_in_2 содержат поля, которые есть в fs1/fs2 но НЕ в другом наборе. Если одно поле есть в extra_in_1 но нет в extra_in_2, это значит оно есть в fs1 но не в fs2 — а при общем хвосте это противоречие (оба типа должны иметь одинаковые named fields + shared tail).

## S2: Non-Exhaustive Pattern Fallback (HIGH)

**Location:** `synoema-core/src/core_ir.rs`, `synoema-core/src/desugar.rs`

**Current behavior:** `build_equation_chain` при пустых equations возвращает `Lit(Int(0))`.

**Required behavior:** Возвращать `CoreExpr::RuntimeError(String)` с сообщением "non-exhaustive pattern match". Interpreter и JIT должны обрабатывать этот вариант.

## S3: Constructor Pattern Type Checking (MEDIUM)

**Location:** `synoema-types/src/infer.rs`, `infer_pattern`

**Current behavior:** `Pat::Con(name, sub_pats)` создаёт fresh type variable, игнорируя тип конструктора.

**Required behavior:** Lookup конструктора в env, instantiate его тип, unify аргументы sub-patterns с параметрами конструктора, вернуть result type.
