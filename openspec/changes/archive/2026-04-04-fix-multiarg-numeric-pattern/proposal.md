# Fix: Multi-arg numeric pattern matching ignores actual value

## Problem

When a function has multiple arguments and the first clause uses a numeric literal pattern (e.g., `f 0 xs = ...`), the numeric pattern matches ALL calls regardless of the actual first argument value. Single-argument numeric patterns work correctly.

## Reproduction

```
f 0 xs = "first"
f n xs = "second"
main = f 3 [1 2]
-- Returns "first" (WRONG), expected "second"
```

Single-arg works correctly:
```
f 0 = "zero"
f n = "other"
main = f 3
-- Returns "other" (correct)
```

## Expected behavior

`f 3 [1 2]` should match the second clause and return `"second"`, since `3 != 0`.

## Area

Parser/desugar layer. Likely in `lang/crates/synoema-core/src/desugar.rs` or `lang/crates/synoema-parser/`. The multi-arg desugaring path probably drops or ignores the numeric literal constraint when generating match arms.

## Severity

High -- pattern matching correctness is fundamental; silent wrong results are worse than crashes.
