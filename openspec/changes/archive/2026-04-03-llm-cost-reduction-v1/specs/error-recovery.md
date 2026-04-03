---
id: error-recovery
type: spec
status: done
---

# Spec: Error Recovery

## Requirement

Parser and type checker collect ALL errors in one pass instead of stopping at the first error.

## Current Behavior

```
Source: "foo x = x +\nbar y = y * 2"
Result: Error at line 1: unexpected newline
        (bar never checked)
```

## Target Behavior

```
Source: "foo x = x +\nbar y = y * 2\nbaz = true + 1"
Result: [
  Error at line 1: unexpected newline in expression,
  Error at line 3: type mismatch: Bool vs Int
]
All 3 errors reported in one pass.
```

## Design

### Parser Recovery

On parse error in a declaration:
1. Record the error with span
2. Skip tokens until next declaration boundary (next line at indent 0, or next `mod`/`use`/uppercase identifier at indent 0)
3. Continue parsing subsequent declarations
4. Return `Program` with partial AST + accumulated errors

### Type Checker Recovery

On type error:
1. Record the error with span
2. Assign a fresh "error type" variable to the failed expression
3. Continue checking subsequent declarations
4. Return accumulated errors

### Output Format

- JSON mode (`--errors json`): array of error objects (already supported, extend to array)
- Human mode: all errors printed sequentially with spans

## Acceptance Criteria

- [ ] Parser collects multiple errors across declarations
- [ ] Type checker collects multiple errors across declarations
- [ ] JSON output returns array of all errors
- [ ] Human-readable output shows all errors
- [ ] Existing single-error tests still pass
- [ ] New tests: multi-error source → all errors reported
- [ ] ≥8 tests (parser recovery + type checker recovery)
