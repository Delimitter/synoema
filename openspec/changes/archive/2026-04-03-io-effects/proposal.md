# Proposal: IO/Effects System

## Problem Statement

Synoema имеет IO-функции (print, readline, file_read, tcp_listen, fd_write, ...) как обычные функции без отслеживания побочных эффектов. Проблемы:

1. **Нет различия pure/impure** — type checker не отличает `add 1 2` от `print "hi"`
2. **Нет формализации** — IO-функции разбросаны как ad-hoc builtins без единой системы
3. **LLM не знает, какие функции чистые** — не может оптимизировать/мемоизировать
4. **`<-` не обобщён** — используется только в list comprehensions, не в IO-контексте
5. **Файловый IO есть только в interpreter** — нет системы для расширения в JIT

## Critical Analysis

### Почему НЕ полный effect system

| Подход | Проблема для Synoema |
|--------|---------------------|
| Haskell IO monad | Тяжёлый синтаксис (>>=, do-notation), overhead для LLM |
| Algebraic effects (Eff/Koka) | Требует handler stacks, runtime overhead, +3000 LOC |
| Free monad | GC-dependent, несовместимо с arena |
| Capability-based | Слишком инвазивно для существующего кода |

### Почему IO type + effect annotation

1. **`IO a` — тип-обёртка** — минимально инвазивный, 1 BPE token для `IO`
2. **Separation of concerns** — type checker отслеживает, где эффекты
3. **Существующий код не ломается** — opt-in, pure функции без аннотаций работают как раньше
4. **LLM понимает паттерн** — `IO String` знаком из Haskell, минимальный learning curve
5. **Совместимо с arena** — IO-значения не аллоцируются, это compile-time маркер

### Scope

**Формализация existing IO + effect tracking в type checker.**
- НЕ добавляем новый IO (файлы, сеть — уже есть в Phase 18)
- НЕ добавляем do-notation или monadic bind
- НЕ меняем runtime / codegen

**Конкретно:**
1. `IO` как тип-конструктор в type system
2. Effect tracking — pure-функция не может вызвать IO-функцию
3. Аннотация builtins: `print : a -> IO ()`, `readline : IO String`, `file_read : String -> IO String`
4. `main` имплицитно `IO`-контекст (любые эффекты разрешены в main)
5. Обновить документацию (docs/llm/io.md, language reference)

## BPE Analysis

| Токен | BPE tokens |
|-------|-----------|
| `IO` | 1 ✓ |
| `@io` | 2 ✗ (нарушает правило) |
| `do` | 1 ✓ (future: do-blocks) |

**Решение:** `IO` как тип-конструктор, НЕ `@io` как аннотация. Отступаем от DEVELOPMENT_GUIDE Phase 9.6, т.к. `@io` = 2 BPE-токена.

## Success Criteria

- [ ] `IO a` парсится как тип-конструктор
- [ ] `print : a -> IO ()` type-checks корректно
- [ ] Pure-функция, вызывающая `print` внутри, получает ошибку типизации
- [ ] `main` автоматически IO-контекст
- [ ] Все IO-builtins (print, readline, file_read, tcp_listen, ...) аннотированы `IO`
- [ ] Существующие программы без IO работают без изменений
- [ ] 0 failures, 0 warnings, все тесты проходят
- [ ] Новые тесты для IO-типизации (≥12)
- [ ] docs/llm/io.md обновлён
