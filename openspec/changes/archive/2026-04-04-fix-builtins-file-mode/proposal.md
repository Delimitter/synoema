# Fix: Several builtins missing in run/jit file mode

## Problem

Builtins `foldl`, `zip`, `index`, `take`, `drop`, `str_length`, and `reverse` are registered in `eval` mode but not in `run`/`jit` file mode, causing "Undefined variable" errors.

## Reproduction

File `test.sno`:
```
main = foldl (\a x -> a + x) 0 [1 2 3]
```

- `eval "foldl (\a x -> a + x) 0 [1 2 3]"` -- returns `6` (correct)
- `run test.sno` -- error: "Undefined variable: foldl"
- `jit test.sno` -- error: "Undefined variable: foldl"

Same issue for `zip`, `index`, `take`, `drop`, `str_length`, `reverse`.

## Expected behavior

All builtins available in `eval` mode should also be available in `run` and `jit` modes.

## Area

Runtime environment setup (`lang/crates/synoema-eval/` or `lang/crates/synoema-repl/`). The `eval` path and the `run`/`jit` path use different environment initialization code; the file-mode path omits some builtin registrations.

## Severity

High -- core list operations are unusable in file mode, forcing users to reimplement standard functions.
