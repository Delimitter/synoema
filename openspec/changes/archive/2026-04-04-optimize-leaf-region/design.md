# Design: Leaf Region Optimization

## Analysis Function

Add `fn needs_heap(expr: &CoreExpr) -> bool` in `compiler.rs` that recursively walks the Core IR tree. Returns `true` if any node could trigger arena allocation.

### Heap-allocating nodes (returns true)

- `MkList(...)` — cons/nil chain
- `MkClosure { .. }` — closure allocation
- `Record(...)` — record allocation
- `RecordUpdate { .. }` — record clone + update
- `Con(name)` — data constructor application (may allocate)
- `Lit(Str(...))` — string literal (calls synoema_str_new)
- `Scope(...)` / `Spawn(...)` — concurrency primitives

### Non-allocating nodes (recurse into children)

- `Var`, `Lit(Int|Bool)`, `PrimOp`, `RuntimeError` — pure values
- `App`, `Lam`, `Let`, `LetRec` — recurse
- `Case` — recurse into scrutinee + all alt bodies
- `FieldAccess` — recurse into base expr
- `Region` ��� the wrapper itself; recurse into body

### Exhaustive match

All CoreExpr variants covered. Adding a new variant forces a compile error.

## Integration Point

In `compile_function()`, wrap the region_enter emission conditionally based on `needs_heap(inner)`. Similarly, in the TCO back-edge, conditionally emit region_exit.

## TcoContext Change

Added `emit_regions: bool` field to pass the flag through to TCO back-edge code.
