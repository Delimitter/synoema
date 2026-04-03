---
id: proposal
type: proposal
status: done
---

# Proposal: Region Inference — Automatic Memory Reclamation for JIT

## Problem Statement

Synoema JIT использует единый 8 MB bump arena с ручным `arena_save` / `arena_restore`. Это работает для "run and discard" сценариев, но:

1. **Серверные циклы требуют ручных вызовов** `arena_save` / `arena_restore` — пользователь (LLM) должен знать и правильно вставлять эти вызовы
2. **Промежуточные аллокации не освобождаются** — `let x = [1..10000] in length x` аллоцирует 10000 ListNode, хотя результат — одно число
3. **Tail-recursive loops накапливают мусор** — каждая итерация аллоцирует, но ничего не освобождается до arena_reset

## Solution

Автоматическое управление регионами на двух уровнях:

### Уровень 1: TCO Auto-Regions (~50 LOC)

Tail-recursive функции уже компилируются как циклы (TcoContext). Добавляем:
- `arena_save()` на входе в цикл
- `arena_restore(saved)` перед каждым back-edge (jump to loop header)

Каждая итерация цикла автоматически освобождает heap. Нулевые изменения в Core IR, нулевые изменения в языке. Чистая оптимизация в codegen.

### Уровень 2: Escape Analysis + Region Insertion (~300 LOC)

Анализ Core IR для автоматической идентификации scopes где heap можно освободить:

1. **Escape analysis pass**: для каждого `let x = e1 in e2` определить, утекает ли `x` из `e2`
2. **Region insertion**: обернуть non-escaping let-bindings в `RegionEnter`/`RegionExit`
3. **Codegen**: emit `arena_save()`/`arena_restore()` для region markers

## Scope

| # | Что | LOC | Crate |
|---|-----|-----|-------|
| 1 | Runtime: region stack (multi-arena) | ~60 | synoema-codegen |
| 2 | TCO auto-regions в compiler.rs | ~40 | synoema-codegen |
| 3 | Escape analysis pass | ~120 | synoema-core |
| 4 | Region annotation pass | ~80 | synoema-core |
| 5 | Codegen для RegionEnter/RegionExit | ~40 | synoema-codegen |
| 6 | Tests + docs | ~100 | all |

**Total: ~440 LOC, 0 новых зависимостей**

## Key Simplifications (vs MLKit)

- **Нет мутации** — все значения immutable → нет tracking mutations
- **Нет region polymorphism** — не нужны region-параметризованные типы
- **Нет region types** — регионы не видны пользователю, чисто internal optimization
- **Используем existing arena** — arena_save/restore уже работают, region inference просто автоматизирует их вставку

## Success Criteria

- [ ] Tail-recursive loops автоматически освобождают heap каждую итерацию
- [ ] `let x = [1..10000] in length x` не держит 10000 nodes до arena_reset
- [ ] Server loops работают с O(max_request) памятью без ручных arena_save/restore
- [ ] Нет регрессии: все 771 тестов зелёные, 0 warnings
- [ ] ≥15 новых тестов
- [ ] JIT performance: нет деградации на существующих бенчмарках

## Non-Goals

- Region polymorphism (типы не меняются)
- User-visible region annotations (пользователь не должен знать о регионах)
- Изменение interpreter'а (Rust RAII достаточен)
- GC, reference counting
