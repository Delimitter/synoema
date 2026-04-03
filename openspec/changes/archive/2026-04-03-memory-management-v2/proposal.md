---
id: proposal
type: proposal
status: done
---

# Proposal: Memory Management v2 — Streaming I/O, Arena Hardening, Server Safety

## Problem Statement

Synoema использует 8 MB bump arena для JIT и Rust heap для interpreter. Это оптимально для основного use case (LLM code generation: short scripts), но имеет три конкретных проблемы:

### 1. Большие файлы невозможно обработать без загрузки целиком

```sno
content = file_read "dataset.csv"   -- 5 GB → 5 GB String в памяти
-- Нет fd_open для файлов → нет построчной обработки
-- fd_readline есть, но только для fd_popen (процессы)
```

`fd_open` для файлов **отсутствует**. Единственный способ читать файл — `file_read` (загрузка целиком).

### 2. Arena overflow молча утекает память

При превышении 8 MB arena JIT fallback'ит на system malloc:
- Аллокация не отслеживается
- `arena_reset()` её не освобождает
- Никакого предупреждения
- Long-running JIT программы постепенно текут

### 3. JIT не подходит для серверов

Нет механизма per-request cleanup в JIT. Арена растёт монотонно внутри одного запуска, `arena_reset()` вызывается только после завершения `main()`. Для HTTP-сервера работающего в JIT это значит: 8 MB за время жизни сервера, потом overflow → leak.

## Scope

Четыре точечных улучшения. Не меняют парадигму (нет GC, нет RC), а устраняют конкретные проблемы.

| # | Что | LOC | Crate |
|---|-----|-----|-------|
| 1 | `fd_open` / `fd_open_write` для файлов | ~50 | synoema-eval |
| 2 | Arena overflow warning + tracking | ~40 | synoema-codegen |
| 3 | `arena_save` / `arena_restore` (per-scope reset) | ~30 | synoema-codegen |
| 4 | Overflow cleanup при `arena_reset` | ~30 | synoema-codegen |

**Total: ~150 LOC, 0 новых зависимостей**

## Why NOT GC / Reference Counting / Region Inference

| Подход | Overhead | Effort | Оправдан? |
|--------|----------|--------|-----------|
| Tracing GC | Pause latency, противоречит "4.4× faster than Python" | 1-2 мес | Нет |
| Ref counting | +8 bytes per object, atomic ops для threads | 2-4 мес | Нет |
| Region inference | Академическая сложность (MLKit) | 3-6 мес | Нет |
| **Точечные fixes** | **Нулевой** | **2-3 дня** | **Да** |

Primary use case (LLM code gen) работает идеально с текущей моделью. Серверы работают через interpreter (Rust heap + auto drop). Большие файлы решаются streaming API.

## Success Criteria

- [ ] `fd_open "file.csv"` возвращает fd для построчного чтения
- [ ] `fd_open_write "output.txt"` возвращает fd для записи
- [ ] Существующий `fd_readline` / `fd_write` / `fd_close` работают с файловыми fd
- [ ] Arena overflow → `eprintln!` warning с размером overflow
- [ ] `arena_save()` / `arena_restore()` доступны из runtime FFI
- [ ] `arena_reset()` освобождает tracked overflow allocations
- [ ] Пример: построчная обработка файла 100+ MB без OOM
- [ ] ≥12 тестов, `cargo test` clean, 0 warnings
- [ ] `docs/llm/synoema.md` обновлена (fd_open, fd_open_write)

## Non-Goals

- Garbage collector (не нужен для target use cases)
- Reference counting (overhead не оправдан)
- Изменение Value representation в interpreter
- WASM target (отдельная задача)
- Автоматический streaming (пользователь выбирает fd_open vs file_read)
