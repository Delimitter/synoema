# Delta Spec: Automation Bug Fixes

## Fix 1: Pattern Matching Variable Binding Collision

**Affected:** `synoema-eval` crate, `apply()` function
**Behavior before:** When a multi-equation function matches the first argument against multiple equations, variable bindings from all matching equations are merged into a single `local` environment. Later equations' bindings overwrite earlier ones.
**Behavior after:** Each matching equation retains its own independent bindings. The `remaining` equations carry per-equation binding snapshots, not shared state.

## Fix 2: `not` and `odd` Builtins in Type Checker

**Affected:** `synoema-types` crate, `builtin_env()` function
**Behavior before:** `not` and `odd` are registered in the evaluator but missing from the type checker, causing "Undefined variable" errors.
**Behavior after:** Both are registered in `builtin_env()`:
- `not : Bool -> Bool`
- `odd : Int -> Bool`

## Fix 3: Nested List Patterns in Constructor Arguments

**Affected:** `synoema-parser` crate, `parse_pattern()` function
**Behavior before:** Constructor argument loop only accepts `LowerId`, `Underscore`, literals, and `LParen`. `Token::LBracket` (`[`) is not accepted, so `(MkStack [])` fails.
**Behavior after:** `Token::LBracket` is accepted in the constructor argument loop, enabling list patterns as constructor arguments.
