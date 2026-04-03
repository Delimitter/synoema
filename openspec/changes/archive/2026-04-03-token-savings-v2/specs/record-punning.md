# Spec: Record Punning

## Syntax

```
record-field = lower-ident "=" expr    -- explicit
             | lower-ident             -- punned: {x} ≡ {x = x}
```

## Semantics

`{x, y, z = expr}` desugars to `{x = x, y = y, z = expr}` at parse time.

## Pattern Punning

`{x, y}` in pattern position desugars to `{x = x, y = y}`.

## Examples

```sno
point x y = {x, y}                    -- record punning
origin = {x = 0, y = 0}               -- explicit (still works)
mixed  = {x, y, sum = x + y}          -- mixed

dist_sq {x, y} = x * x + y * y        -- pattern punning
```

## Constraints

- Punned field name MUST be a valid `lowerId`
- Punned name resolves in current scope (like any variable reference)
- No ambiguity: `{x}` is always record punning, never a set or block
