# Fix: Singleton list pattern `[x]` not supported

## Problem

The parser does not recognize `[x]` as a valid pattern in function clause heads. It produces a parse error instead of desugaring to the cons pattern `(x : [])`.

## Reproduction

```
head [x] = x
head (x : _) = x
main = head [42]
-- Parser error: "Expected RBracket, got 'x'"
```

## Expected behavior

`[x]` in pattern position should parse and desugar to `(x : [])`, matching a list with exactly one element. By extension, `[x y]` should desugar to `(x : (y : []))`, etc.

## Area

Parser (`lang/crates/synoema-parser/src/parser.rs`). The pattern-parsing code path for `[...]` likely only handles empty `[]` (nil) and does not enter a loop to collect element sub-patterns.

## Severity

Medium -- workaround exists (use explicit cons pattern `(x : [])`), but the sugar form is idiomatic and expected.
