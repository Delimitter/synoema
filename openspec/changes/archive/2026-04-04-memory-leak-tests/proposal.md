# Proposal: Memory Leak Tests

## Problem
При выполнении тестов происходит утилизация памяти (arena allocator, region stack, overflow allocs). Нет тестов, которые целенаправленно проверяют:
1. Отсутствие утечек памяти после arena_reset
2. Корректность region_enter/region_exit (баланс вложенности)
3. Очистку overflow allocations при arena_reset
4. Что существующие тесты не утекают память между запусками

## Context
- Arena: 8 MB bump allocator, thread_local, region stack (depth 64)
- Region inference: escape analysis в optimize.rs → CoreExpr::Region
- Overflow: fallback на system malloc, tracked в overflow_allocs
- Известные "утечки": Box::leak для string literals (compile-time), channels (Phase C)

## Scope
1. Добавить тест-утилиты для замера arena offset до/после выполнения
2. Написать тесты на утечки: region balance, overflow cleanup, arena reset completeness
3. Проверить все существующие JIT-тесты на утечки (arena offset == 0 после reset)
4. Исправить найденные утечки

## Non-goals
- Не менять архитектуру arena allocator
- Не трогать Box::leak для string literals (compile-time, не runtime leak)
- Не добавлять внешние зависимости (valgrind, miri в CI)
