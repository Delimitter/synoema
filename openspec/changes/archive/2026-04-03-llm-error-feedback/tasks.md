---
id: tasks
type: tasks
status: done
---

# Tasks: LLM Error Feedback System

## Checklist

- [x] **T1: Extend Diagnostic struct**
  - `synoema-diagnostic/src/lib.rs`
  - Добавить поля: `llm_hint: Option<String>`, `fixability: Option<Fixability>`, `did_you_mean: Option<String>`
  - Добавить enum `Fixability { Trivial, Easy, Medium, Hard }`
  - Обновить JSON renderer для новых полей
  - Обновить human renderer (показывать hint если есть)
  - ~40 LOC

- [x] **T2: Error enrichment engine**
  - `synoema-diagnostic/src/lib.rs`
  - Функция `enrich_diagnostic(diag: &mut Diagnostic)`
  - Match по `diag.code` → set llm_hint, fixability, did_you_mean
  - Покрыть top-12 ошибок: type_mismatch, type_arity, unbound_variable, infinite_type, pattern_mismatch, unexpected_token, expected_expression, unterminated_string, no_match, div_zero, linear_unused, linear_duplicate, indentation
  - ~100 LOC

- [x] **T3: Did-You-Mean для syntax errors**
  - `synoema-diagnostic/src/lib.rs` (enrichment)
  - 4 правила: if→?, commas→spaces, ->→\->, return→expr
  - Detection: по тексту ошибки
  - ~50 LOC

- [x] **T4: Indentation error diagnostics**
  - `synoema-eval/src/lib.rs` — detection at parse_err conversion
  - Новый error code: `PARSE_INDENTATION`
  - Включает контекст: column info + offside rule hint
  - ~20 LOC

- [x] **T5: Wire enrichment into eval pipeline**
  - `synoema-eval/src/lib.rs` — enrich_diagnostic() в parse_err, type_err, eval_err
  - `synoema-repl/src/main.rs` — enrich_diagnostic() в print_diag
  - JSON output включает новые поля
  - ~15 LOC

- [x] **T6: Feedback loop script**
  - `tools/llm/feedback_loop.py`
  - Pipeline: prompt → LLM (API) → .sno → cargo run --errors json → parse errors → format for LLM → retry
  - Configurable: max_retries (default 3), temperature_decay (1.0 → 0.5 → 0.2)
  - Support: OpenAI API, Anthropic API (Claude)
  - ~200 LOC

- [x] **T7: Tests**
  - `synoema-diagnostic` tests: enrichment для каждого top-12 error (12 tests)
  - Did-you-mean tests (3 tests)
  - JSON output tests (2 tests)
  - Human renderer tests (1 test)
  - Builder/helper tests (4 tests)
  - Integration tests in synoema-eval (3 tests)
  - 25 новых тестов total

- [x] **T8: Documentation**
  - `docs/llm/error-feedback.md` — формат ошибок, таблица llm_hint, примеры
  - Updated `docs/llm/synoema.md` — секция "Error handling"
  - Updated `context/PROJECT_STATE.md`

- [x] **T9: Final verification**
  - `cargo test` — 797 tests pass, 0 failures, 0 warnings
  - `cargo build` — 0 warnings
  - All enrichment paths tested
