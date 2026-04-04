# Fix: JIT returns raw tagged pointer values for list results

## Problem

When a program returns a list value, the JIT backend prints the raw tagged pointer integer instead of a formatted list. The interpreter correctly formats the output.

## Reproduction

```
main = [1 2 3]
```

- `run` (interpreter): prints `[1 2 3]` (correct)
- `jit`: prints `50767855632` (raw tagged pointer)

Also affects `tail`:
```
main = tail [1 2 3]
-- JIT prints raw pointer instead of [2 3]
```

## Expected behavior

JIT mode should format list results identically to the interpreter: `[1 2 3]`, `[2 3]`, etc.

## Area

JIT codegen display logic (`lang/crates/synoema-codegen/`). The result printing code after JIT execution does not walk the tagged pointer list structure to produce human-readable output. The tagged pointer ABI decoding for cons cells needs a display formatter.

## Severity

Medium -- JIT produces correct values internally but the output is unusable for list-returning programs.
