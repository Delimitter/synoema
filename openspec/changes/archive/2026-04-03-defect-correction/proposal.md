# Proposal: Defect Correction — Type System Soundness

## Problem Statement

Аудит кодовой базы выявил 3 дефекта, влияющих на корректность системы типов и runtime-безопасность:

1. **CRITICAL** — Row polymorphism: при `r1 == r2` поля, присутствующие в обоих наборах `extra_in_1`/`extra_in_2`, не унифицируются. Позволяет `{x: Int, y: Bool | r}` пройти унификацию с `{x: Int, y: String | r}`.
2. **HIGH** — Non-exhaustive patterns: `build_equation_chain` возвращает `Lit(Int(0))` как fallback вместо runtime error.
3. **MEDIUM** — Constructor patterns: `infer_pattern` для `Pat::Con` создаёт fresh type variable вместо lookup типа конструктора из env.

## Scope

- 3 дефекта, 5 файлов, ~6-8 новых тестов
- Не затрагивает JIT ABI, arena, lexer, parser
- Обратно совместимо: корректные программы не меняют поведение

## Success Criteria

- Все 702+ тестов зелёные
- 0 warnings
- Новые тесты покрывают каждый дефект (и positive, и negative cases)
