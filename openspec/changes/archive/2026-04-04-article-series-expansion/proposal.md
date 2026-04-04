# Proposal: Article Series Expansion

## Problem

Три статьи серии "Token Economics of Code" опубликованы на Medium (теоретическая база). Статьи 4-7 готовы, но содержат устаревшие числа (264 теста → 890+, 7055 LOC → ~12000). Серия не покрывает практические данные бенчмарков — ключевой контент для привлечения аудитории разработчиков.

## Goal

1. Обновить статьи 4-7 с актуальными данными проекта
2. Создать 4 новых статьи (#8-#11), основанных на реальных benchmark-данных
3. Адаптировать формат под Medium и dev.to (две версии каждой статьи)
4. Обновить series_plan.md с полным планом серии из 11 статей

## Scope

### Обновление существующих статей (4-7)
- #4 Compilation/JIT: числа, LOC, test count, актуальные бенчмарки
- #5 Hindley-Milner: добавить практические примеры, связь с Phase C бенчмарками
- #6 Launch: 890+ тестов, ~12K LOC, все новые фичи (prelude, MCP, region inference, record update, modules)
- #7 Future: отметить что из "будущего" уже реализовано, MCP integration

### Новые статьи (8-11)
- #8 Token Efficiency: 16 задач × 5 языков, полный breakdown Phase A данных
- #9 Runtime Benchmark: JIT vs interpreters, Phase B данные
- #10 LLM Generation: 10 моделей × 9 задач, Phase C (план — данных пока нет)
- #11 Cost Calculator: практический расчёт экономии для команд

### Обновление series_plan.md
- Расширить до 11 статей
- Новый порядок публикации: данные (#8) → решение (#4) → runtime (#9) → типы (#5) → LLM (#10) → калькулятор (#11) → launch (#6) → vision (#7)

## Out of scope
- Запуск бенчмарков (стабилизация benchmark suite — отдельный change)
- Публикация на Хабр (RU-версии — следующий шаг)
- Создание интерактивного калькулятора (только статья с формулами)

## Open decisions
- Бенчмарки runtime нестабильны → #9 будет содержать placeholder-секцию "[Results from stable benchmark run]"
- Phase C (LLM generation) не запускался → #10 будет template с методологией
