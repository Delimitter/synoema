---
id: design
type: design
status: done
---

# Design: LLM Minified Reference

## Key decisions

### D1: Markdown with fenced code blocks
LLMs parse markdown well. Fenced ```sno blocks give syntax hints.
Alternative considered: plain text — rejected, harder for LLM to isolate code.

### D2: Contrast-first for known constructs
Open with "no if/let/def" — this prevents the #1 class of generation errors.
LLMs remap unknown syntax via similarity; we must override defaults explicitly.

### D3: One-liner examples only
Each construct gets a single working code snippet, not a full program.
Exception: pattern matching needs 2-3 lines to show multi-equation.

### D4: Operator table stays
Operators are the densest info. A table beats prose for token efficiency.
Remove BPE column (irrelevant to code generation), keep: op | meaning | prec.

### D5: No type inference rules
Algorithm W rules are noise for code generation. Include only: "types inferred,
annotations optional, use `name : Type -> Type` for docs."

### D6: Gotchas section
Common traps for LLMs:
- List literals: `[1 2 3]` not `[1, 2, 3]`
- Cons pattern: `(x:xs)` not `x:xs` (parens required in patterns)
- Conditional: `? cond -> then : else` — all on one line or indented
- String concat: `++` not `+`
- No `return` — last expr is the value

### D7: Output path
`docs/llm/synoema.md` — separate from specs/ (LLM tooling directory).
Create `docs/llm/` directory.

## Section order rationale
Functions first (after axioms) because function definition is the most-used construct.
ADTs before typeclasses (dependency order). IO last (least common in generated code).
"Absent" section near end — after LLM has mental model, the negatives are memorable.
