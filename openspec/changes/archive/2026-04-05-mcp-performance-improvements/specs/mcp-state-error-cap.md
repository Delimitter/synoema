# Spec: State Error Cap

## Capability

Ограничение `last_error` в StateTracker до 500 символов.

## Current Behavior

- `last_error: Option<String>` хранит полный текст ошибки без ограничений
- Длинные stack traces (1000+ chars) раздувают baseline_context

## New Behavior

- При записи `last_error` обрезается до 500 символов с добавлением `... (truncated)`
- baseline_context гарантированно не превышает разумный размер

## Changes

### state.rs

1. В `on_tool_result` при установке last_error — truncate до 500 chars
2. Добавить тест: ошибка >500 символов обрезается

## Acceptance Criteria

- last_error.len() ≤ 515 (500 + "... (truncated)")
- Существующие тесты проходят
