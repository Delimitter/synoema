# Spec: mcp-context-tool

## Capability

Инструмент `get_context` возвращает baseline контекст, оптимизированный для текущего состояния разработки.

## Behavior

### State: Create

Полный набор для написания кода:
- LLM quick reference (≤1500 tokens, из `synoema://spec/llm_ref`)
- Список примеров (из `synoema://examples`)
- Подсказка: "Use `eval` to test expressions, `typecheck` to verify types"

### State: Check

Фокус на исправлении ошибок:
- Последняя ошибка (JSON diagnostic, если есть)
- Подсказка с fixability и did_you_mean (если есть)
- Краткий список типичных ошибок и их фиксов

### State: Run

Минимальный контекст (программа работает):
- Подсказка: "Program running. Use `eval` to test changes, `run` to re-execute"
- Совет по оптимизации (если был runtime warning)

### State: Debug

Контекст для отладки:
- Последняя ошибка (JSON diagnostic)
- Подсказка: "Use `search_code` to find related code, `get_context_for_edit` to inspect"
- Список доступных debug-инструментов

## API

```json
{
  "name": "get_context",
  "description": "Get state-aware baseline context for current development phase (Create/Check/Run/Debug)",
  "inputSchema": {
    "type": "object",
    "properties": {},
    "required": []
  }
}
```

## Content Budget

- Create: ≤1800 tokens (полная спека)
- Check: ≤600 tokens (ошибка + hints)
- Run: ≤200 tokens (minimal)
- Debug: ≤800 tokens (ошибка + tools)
