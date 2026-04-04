# Proposal: Fix Benchmark Suite & Fill Article Data

## Problem

Статьи #8-#11 содержат [PLACEHOLDER] данные. Нужно:
1. Запустить Phase A (token efficiency) — все 16 задач × 5 языков
2. Диагностировать нестабильность Phase B (runtime) — в разных прогонах Synoema то быстрее, то медленнее Python
3. Запустить стабильный Phase B
4. Заполнить placeholder-данные в статьях реальными числами

## Root Cause Analysis (Phase B instability)

Из существующих результатов:
- run_003: Synoema = baseline, C++ "2.5x slower" — это НЕВЕРНО, вероятно Synoema JIT запускался из debug build
- run_004: factorial Synoema 4.3ms, C++ 1.3ms — реалистично (JIT = 3.2× медленнее C++)
- run_005: fibonacci Synoema 22ms, Python 14ms — Synoema медленнее Python, возможно JIT compile overhead
- run_006/007: Synoema 2700-3039ms — что-то сильно не так, вероятно fallback на `cargo run` вместо release binary

Гипотеза: runtime.rs ищет release binary, но при отсутствии fallback-ит на debug build или `cargo run`, что добавляет секунды overhead.

## Scope

1. Убедиться что release binary существует
2. Запустить Phase A (все 16 задач)
3. Запустить Phase B (runtime) с release binary
4. Заполнить данные в статьях #8, #9, #11
5. Phase C (LLM generation) — НЕ в scope (требует API key)
