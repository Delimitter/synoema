# Spec: mcp-state-tracker

## Capability

MCP-сервер автоматически определяет текущее состояние разработки (Create/Check/Run/Debug) по вызовам инструментов и их результатам.

## States

| State | Описание | Baseline context |
|-------|---------|-----------------|
| Create | Написание нового кода, исследование API | llm_ref + stdlib catalog + examples index |
| Check | Проверка типов, исправление ошибок компиляции | last error (JSON) + type hints + fixability |
| Run | Выполнение программы, анализ вывода | minimal (result summary) |
| Debug | Отладка runtime-ошибок, анализ поведения | error + span context + arena stats |

## Transitions

| From | Event | To |
|------|-------|----|
| * (initial) | — | Create |
| Create | eval success | Create |
| Create | typecheck success | Check |
| Create | eval error | Check |
| Create | typecheck error | Check |
| Create | run success | Run |
| Create | run error | Debug |
| Check | eval success | Create |
| Check | typecheck success | Check |
| Check | run success | Run |
| Check | run error | Debug |
| Run | eval/typecheck call | Create |
| Run | run error | Debug |
| Debug | eval success | Create |
| Debug | search_code/get_context | Create |
| Debug | run success | Run |

## Tool: get_state

- Возвращает текущее состояние и историю последних 5 переходов
- Формат: JSON `{ "state": "Create", "history": [...] }`

## Tool: get_context

- Возвращает baseline контекст для текущего состояния
- Содержимое зависит от state (см. таблицу States)
- Формат: MCP text content (markdown)

## Invariants

- StateTracker не влияет на результаты существующих tools
- Все существующие API остаются без изменений
- State обновляется ПОСЛЕ выполнения tool call (не до)
- Default state при инициализации: Create
