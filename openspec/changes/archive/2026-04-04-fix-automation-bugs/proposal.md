# Proposal: Fix Automation Bugs

## Problem
20 tool-driver examples were created and 8 language/compiler issues were discovered. 3 of these are actual compiler bugs that produce wrong results or block valid code:

1. **3-equation pattern matching**: Functions with 3+ equations and mixed literal/variable patterns return wrong values. `merge [3] []` returns `[]` instead of `[3]` when there are 3 equations. Root cause: variable bindings from later matching equations overwrite earlier ones in eval.rs lines 513-526.
2. **`not`/`odd` missing from type checker**: eval.rs registers both builtins, but infer.rs doesn't — so the type checker rejects them as "undefined variable".
3. **Nested list patterns in constructors**: `(MkStack [])` fails to parse because the constructor argument loop (parser.rs lines 572-582) doesn't accept `Token::LBracket`.

## Scope
- Fix the pattern matching variable-binding collision in eval.rs
- Add `not` and `odd` to the type checker in infer.rs
- Add `Token::LBracket` to the constructor pattern argument loop in parser.rs
- Add regression tests for each fix
- Update documentation (test counts)

## Out of Scope
- `++` type ambiguity (complex type system change, needs separate design)
- `json_encode` in interpreter (documented as "JIT only")
- `:` and `[f x]` behavior (by design, documented gotchas)
