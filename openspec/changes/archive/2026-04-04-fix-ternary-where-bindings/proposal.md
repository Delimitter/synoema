# Fix: Where-bindings inside ternary branches fail to parse

## Problem

Where-bindings (`x = expr; x`) work at function body level but fail when used inside a ternary (`? cond -> ...`) branch. The parser sees the `=` as an unexpected assignment token.

## Reproduction

```
main = ? true -> x = 42; x
-- Error: "Expected expression, got Assign"
```

Works at top level:
```
main = x = 42; x
-- Returns 42
```

## Expected behavior

Where-bindings should be valid expressions in any expression position, including ternary branches. `? true -> x = 42; x` should evaluate to `42`.

## Area

Parser (`lang/crates/synoema-parser/src/parser.rs`). The ternary branch parsing likely calls a restricted expression parser that does not accept `let`/where-binding syntax.

## Severity

Medium -- limits composability of where-bindings; users must hoist bindings above the ternary.
