---
id: proposal
type: proposal
status: draft
---

# Proposal: LLM Error Feedback System

## Problem Statement

Synoema имеет зрелую систему диагностики (structured JSON, multi-error recovery, stable error codes), но она оптимизирована для **человека-разработчика**, не для **LLM в feedback loop**.

Типичный LLM pipeline: generate → check → error? → **feed error back to LLM** → retry. Текущие ошибки не содержат:
- `llm_hint` — actionable instruction для исправления
- `fixability` — оценка сложности исправления
- `suggested_fix` — конкретное предложение
- `did_you_mean` — для частых ошибок (if/then/else → ?/->/:)

### Самые частые ошибки LLM (по taxonomy анализу)

**Tier 1 — >50% всех ошибок LLM:**

| Ошибка | Freq | Fixability | Diagnostic Quality |
|--------|------|------------|-------------------|
| TYPE_MISMATCH | Очень высокая | Тривиальная | ⭐⭐⭐⭐⭐ Отличная |
| TYPE_ARITY | Очень высокая | Тривиальная | ⭐⭐⭐⭐⭐ Отличная |
| TYPE_UNBOUND_VAR | Высокая | Лёгкая | ⭐⭐⭐⭐ Хорошая |
| Indentation/offside | Высокая | **Сложная** | ⭐ **Критически плохая** |
| Wrong syntax (commas, if/then) | Высокая | Тривиальная | ⭐⭐⭐ Средняя |

**Tier 2 — 20-50%:**

| Ошибка | Freq | Fixability | Diagnostic Quality |
|--------|------|------------|-------------------|
| EVAL_NO_MATCH | Средняя | Лёгкая | ⭐⭐⭐ Средняя |
| Missing parens in cons | Средняя | Лёгкая | ⭐⭐ Плохая |
| LEX_UNTERMINATED_STRING | Средняя | Тривиальная | ⭐⭐⭐⭐⭐ Отличная |

### Критическая проблема: Offside Rule

Ошибки отступа составляют ~20% syntax errors, но диагностика **не упоминает indentation**:
```
-- Текущее сообщение:
error[unexpected_token] at 5:3: unexpected token

-- Нужно:
error[indentation] at 5:3: expected indent of 4 spaces (got 2);
  check offside rule — inner expressions must be indented further than the enclosing definition
```

## Scope

### Layer 1: LLM-Enriched Diagnostics (~150 LOC Rust)
- Добавить поля `llm_hint`, `fixability`, `did_you_mean` в Diagnostic
- Enrichment rules для top-20 ошибок
- Обратно-совместимо: JSON renderer выводит новые поля

### Layer 2: Indentation Error Improvement (~100 LOC Rust)
- Lexer layout pass: emit specific indentation diagnostics
- Включить текущий и ожидаемый уровень отступа
- Контекст: "inside definition of f starting at line 3"

### Layer 3: Orchestration Script (~200 LOC Python)
- `tools/llm/feedback_loop.py`
- Pipeline: prompt → generate → check → enrich errors → format for LLM → retry
- Configurable: max retries, model, temperature decay

## Success Criteria

- [ ] Top-10 ошибок LLM имеют `llm_hint` с actionable instruction
- [ ] Indentation errors включают expected/actual indent level
- [ ] `did_you_mean` для 5 частых syntax ошибок (if→?, [1,2]→[1 2], etc.)
- [ ] JSON output включает `fixability: "high"|"medium"|"low"`
- [ ] Feedback loop script: 3 retry → ≥80% auto-fix rate для Tier 1 errors
- [ ] `cargo test` clean, 0 warnings
- [ ] Документация: `docs/llm/error-feedback.md`

## Non-Goals

- Stack traces (требует переработки evaluator — отдельная задача)
- Source spans для runtime errors (требует пробрасывания spans через eval)
- IDE integration (LSP — отдельная задача)
