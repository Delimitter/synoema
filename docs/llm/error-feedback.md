# LLM Error Feedback

JSON error output with `--errors json` includes LLM-actionable fields.

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `llm_hint` | `string?` | Actionable fix instruction |
| `fixability` | `"trivial"\|"easy"\|"medium"\|"hard"` | Fix difficulty |
| `did_you_mean` | `string?` | Alternative syntax suggestion |

## Enriched Error Codes

| Code | Fixability | LLM Hint Summary |
|------|-----------|-------------------|
| `type_mismatch` | trivial | Change expression to produce expected type |
| `arity_mismatch` | trivial | Add/remove arguments |
| `unbound_variable` | easy | Check spelling, add parameter/definition |
| `infinite_type` | hard | Break cycle with ADT wrapper |
| `pattern_mismatch` | easy | Check constructor names/arity |
| `unexpected_token` | trivial | Check syntax + did_you_mean |
| `unterminated_string` | trivial | Add closing quote |
| `no_match` | easy | Add catch-all pattern |
| `division_by_zero` | trivial | Guard with conditional |
| `linear_unused` | easy | Use or remove variable |
| `linear_duplicate` | easy | Use exactly once |
| `indentation` | easy | Follow offside rule, 2-space indent |

## Did-You-Mean Rules

| LLM Writes | Suggestion |
|------------|------------|
| `if x then y else z` | `? x -> y : z` |
| `[1, 2, 3]` | `[1 2 3]` (no commas) |
| `return x` | just `x` (expression-based) |
| `x -> y` (lambda) | `\x -> y` (needs backslash) |

## JSON Example

```json
{
  "code": "type_mismatch",
  "severity": "error",
  "message": "expected Int, found String",
  "span": {"line": 3, "col": 14, "end_line": 3, "end_col": 20},
  "notes": ["expected: Int", "found: String"],
  "llm_hint": "Change the expression to produce Int instead of String. Common fixes: type conversion, different operator, or fix the literal value.",
  "fixability": "trivial"
}
```

## Feedback Loop Script

`tools/llm/feedback_loop.py` — generate, check, enrich, retry pipeline.

```bash
python tools/llm/feedback_loop.py --prompt "Write factorial" --provider openai --retries 3 -v
python tools/llm/feedback_loop.py --prompt-file task.txt --provider anthropic
```

Temperature decay: 1.0 -> 0.5 -> 0.2 across retries.
