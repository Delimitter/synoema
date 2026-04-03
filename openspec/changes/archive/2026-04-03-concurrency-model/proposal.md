# Proposal: Concurrency Model for Synoema

## Problem Statement

Synoema не имеет модели конкурентности. При генерации параллельного кода LLM не может гарантировать отсутствие гонок данных. Нужен подход, который:
1. Делает data races невозможными на уровне типов
2. Минимален по синтаксису (BPE-alignment)
3. Совместим с arena allocation и Cranelift JIT
4. Не требует тяжёлого runtime (no tokio, no GC, no scheduler)
5. Удобен для LLM-генерации

## Critical Analysis

### Почему НЕ полная модель акторов/CSP/STM

| Модель | Проблема для Synoema |
|--------|---------------------|
| Actor (BEAM) | Требует per-process GC, preemptive scheduler (~1M LOC runtime) |
| CSP (Go) | Требует scheduler, goroutine stacks, несовместимо с arena |
| STM | Требует GC для version history, конфликтует с `arena_reset()` |
| Rust ownership | Borrow checker слишком сложен для LLM-генерации |

### Почему линейные типы + structured concurrency

1. **Линейные типы** — compile-time, zero runtime cost, 2 простых правила
2. **Structured concurrency** — scope гарантирует, что потоки не переживают родителя
3. **Каналы с copy semantics** — arena-совместимы, нет shared state
4. **Cranelift compiled functions** — безопасно вызывать из нескольких OS threads

### Риски и ограничения

- **Scope**: Линейные типы затрагивают ВЕСЬ type checker (usage counting в каждом scope)
- **Совместимость**: Существующие программы должны продолжать работать (линейность opt-in)
- **Arena threading**: Текущий arena — thread_local, нужны per-thread arenas для JIT
- **Interpreter overhead**: Usage tracking может замедлить eval

## Phased Approach

### Phase A: Linear/Unique Types (TYPE SYSTEM ONLY)
- Добавить `Multiplicity` (One | Many) к `Arrow`
- Синтаксис: `-o` для линейной стрелки (1 BPE token)
- Usage counting в type checker
- Interpreter: assert linearity (runtime check)
- **Без runtime changes, без JIT changes**

### Phase B: Structured Concurrency Runtime
- `scope { ... }` — создание scope
- `spawn expr` — запуск в OS thread внутри scope
- Per-thread arenas
- **Только interpreter**

### Phase C: Channels + JIT
- `chan` — создание типизированного канала
- `send` / `recv` — отправка/получение (copy semantics)
- JIT codegen для spawn/scope/chan

## Scope This Change

**Только Phase A** — линейные типы как фундамент безопасности.
Phases B и C — отдельные changes.

## Success Criteria

- [ ] `f : Int -o Int` — синтаксис линейной стрелки парсится
- [ ] Type checker отклоняет программу, где линейная переменная используется дважды
- [ ] Type checker отклоняет программу, где линейная переменная не используется
- [ ] Существующие программы (без -o) работают без изменений
- [ ] 0 warnings, все существующие тесты проходят
- [ ] Новые тесты для линейности (≥10)
