---
id: design
type: design
status: draft
---

# Design: LLM Error Feedback System

## Layer 1: LLM-Enriched Diagnostics

### Extended Diagnostic Structure

```rust
pub struct Diagnostic {
    pub code: &'static str,
    pub severity: Severity,
    pub message: String,
    pub span: Option<Span>,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
    // NEW:
    pub llm_hint: Option<String>,      // Actionable fix instruction
    pub fixability: Option<Fixability>, // How hard to fix
    pub did_you_mean: Option<String>,   // Alternative syntax
}

pub enum Fixability {
    Trivial,  // Typo, missing delimiter, wrong symbol
    Easy,     // Add argument, fix type, add pattern
    Medium,   // Restructure code, redesign types
    Hard,     // Rethink algorithm, infinite type
}
```

### JSON Output

```json
{
  "code": "type_mismatch",
  "severity": "error",
  "message": "expected Int, found String",
  "span": {"line": 3, "col": 12, "end_col": 18},
  "notes": ["expected: Int", "found: String"],
  "llm_hint": "Function '+' expects Int arguments. Use 'read' to convert String to Int, or use '++' for string concatenation.",
  "fixability": "trivial",
  "did_you_mean": null
}
```

### Enrichment Rules (Top-20 Errors)

```rust
fn enrich_diagnostic(diag: &mut Diagnostic) {
    match diag.code {
        codes::TYPE_MISMATCH => {
            // Extract expected/found from notes
            diag.fixability = Some(Fixability::Trivial);
            diag.llm_hint = Some(format!(
                "Change the expression to produce {} instead of {}. \
                 Common fixes: type conversion, different operator, \
                 or fix the literal value.",
                expected, found
            ));
        }
        codes::TYPE_UNBOUND_VAR => {
            diag.fixability = Some(Fixability::Easy);
            diag.llm_hint = Some(format!(
                "Variable '{}' is not defined. Check spelling, \
                 add it as a function parameter, or define it in a where-block.",
                name
            ));
            // TODO: fuzzy match against known names for did_you_mean
        }
        // ... 18 more rules
    }
}
```

### Syntax Error Did-You-Mean

| LLM Writes | Error | Did-You-Mean |
|-------------|-------|--------------|
| `if x > 0 then y else z` | unexpected `if` | `? x > 0 -> y : z` |
| `[1, 2, 3]` | unexpected `,` | `[1 2 3]` (space-separated) |
| `f x y = ...` without `(` for patterns | unexpected token | `f (x:xs) = ...` (cons needs parens) |
| `x -> y` in lambda | unexpected `->` | `\x -> y` (lambda needs backslash) |
| `return x` | unexpected `return` | Synoema is expression-based: just write `x` |

## Layer 2: Indentation Error Improvement

### Current Problem

Lexer's layout pass converts indentation into `Indent`/`Dedent` tokens, but when these cause parse errors, the error message says "unexpected token" without mentioning indentation.

### Solution

Add indentation context to layout engine:

```rust
struct LayoutState {
    indent_stack: Vec<usize>,  // Stack of indentation levels
    current_indent: usize,     // Current column
}

// When generating Indent/Dedent that will cause parse error:
fn emit_indent_diagnostic(&self, pos: Pos) -> Diagnostic {
    let expected = self.indent_stack.last().copied().unwrap_or(0);
    Diagnostic::error(
        codes::PARSE_INDENTATION,
        format!(
            "indentation error: expected indent of {} (got {})",
            expected, self.current_indent
        ),
    )
    .with_span(Span::new(pos, pos))
    .with_note(format!(
        "inner expressions must be indented further than the \
         enclosing definition (column {})",
        expected
    ))
    .with_llm_hint(
        "Synoema uses the offside rule (like Haskell/Python). \
         Indent the body of a definition further than its name. \
         Use consistent 2-space indentation."
    )
    .with_fixability(Fixability::Easy)
}
```

### New Error Code

Add `PARSE_INDENTATION = "indentation"` to codes module.

## Layer 3: Feedback Loop Orchestration

```
    ┌────────────────────────────────────┐
    │         feedback_loop.py            │
    │                                    │
    │  prompt ──▶ LLM ──▶ .sno ──▶ check│
    │                                │   │
    │            ┌───── error? ◀─────┘   │
    │            │                       │
    │            ▼                       │
    │   format_error_for_llm()          │
    │   - JSON diagnostic               │
    │   - llm_hint included             │
    │   - source context                │
    │   - "Fix this error and           │
    │      regenerate the program"      │
    │            │                       │
    │            ▼                       │
    │   retry with enriched prompt      │
    │   (max 3 retries, temp decay)     │
    └────────────────────────────────────┘
```

### Error Format for LLM

```
The following Synoema program has an error:

```sno
fac 0 = 1
fac n = n * fac (n - 1)
main = fac "five"
```

Error at line 3, column 14:
  type_mismatch: expected Int, found String
  Hint: Function 'fac' expects Int argument. Replace "five" with an integer like 5.
  Fixability: trivial

Please fix the error and output the corrected program.
```
