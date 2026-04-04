# Design: Leaf Region Optimization

## Analysis Function

Add `fn needs_heap(expr: &CoreExpr) -> bool` in `compiler.rs` that recursively walks the Core IR tree. Returns `true` if any node could trigger arena allocation.

### Heap-allocating nodes (returns true)

- `MkList(...)` — cons/nil chain
- `MkClosure { .. }` — closure allocation
- `Record(...)` — record allocation
- `Con(name)` — data constructor application (may allocate)
- `Lit(Str(...))` — string literal (calls synoema_str_new)
- `Scope(...)` / `Spawn(...)` — concurrency primitives
- `StringInterp(...)` — string interpolation (allocates)

### Non-allocating nodes (recurse into children)

- `Var`, `Lit(Int|Bool)`, `PrimOp` — pure values
- `App`, `Lam`, `Let`, `LetRec` — recurse
- `Case` — recurse into scrutinee + all alt bodies
- `FieldAccess` — recurse into base expr
- `Region` — the wrapper itself; recurse into body

### Conservative default

Unknown/new variants → return `true` (assume heap needed).

## Integration Point

In `compile_function()` (compiler.rs:515-527), wrap the region_enter emission:

```
let emit_regions = needs_heap(inner);
if emit_regions {
    // existing region_enter code
}
```

Similarly, in the TCO back-edge (compiler.rs:788-792), conditionally emit region_exit:

```
if emit_regions {
    // existing region_exit code
}
```

The `emit_regions` flag is passed through `TcoContext` as a new field.

## TcoContext Change

```rust
struct TcoContext {
    self_name: String,
    loop_block: Block,
    params: Vec<Variable>,
    emit_regions: bool,  // NEW: skip region calls for leaf functions
}
```

## No Other Changes

- Runtime: untouched
- Core IR: untouched
- Evaluator: untouched (interpreter doesn't use arena regions)
