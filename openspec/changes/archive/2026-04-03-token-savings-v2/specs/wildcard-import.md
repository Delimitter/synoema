# Spec: Wildcard Import

## Syntax

```
use-decl   = "use" upper-ident "(" use-names ")"
use-names  = "*"                           -- wildcard: import all exports
           | lower-ident (lower-ident)*    -- selective: named imports
```

## Semantics

`use Math (*)` imports all function definitions from module `Math`. Equivalent to listing every exported name.

## Examples

```sno
mod Math
  square x = x * x
  cube x = x * x * x
  abs x = ? x < 0 -> 0 - x : x

use Math (*)               -- imports square, cube, abs
-- equivalent to: use Math (square cube abs)

main = square 5 + cube 3
```

## Constraints

- `*` inside `use ... (*)` MUST be the only token — no mixing `use M (* square)`
- Wildcard resolves at module-expansion time (same phase as selective imports)
- If module has no exports, `use M (*)` is a no-op (no error)
- ADT constructors and type aliases defined inside module are NOT imported by wildcard (only function definitions)
