# Design: Self-Recursive TCO in JIT

## Implementation Location

All changes in `lang/crates/synoema-codegen/src/compiler.rs`.

## Key Mechanism

### 1. Thread `self_name` and `entry_block` Through Compilation

Currently `compile_expr()` signature:
```rust
fn compile_expr(builder, vars, vc, funcs, module, ctor_tags, expr) -> Result<Value>
```

Add a new parameter `tco_ctx: Option<TcoContext>`:
```rust
struct TcoContext {
    self_name: String,       // function being compiled
    entry_block: Block,      // entry block to jump back to
    params: Vec<Variable>,   // parameter Variables for reassignment
}
```

Pass `Some(tco_ctx)` only when compiling in tail position. Pass `None` for non-tail sub-expressions.

### 2. Tail Position Propagation

- `compile_function` body: pass `Some(tco_ctx)`
- `Case` branches: propagate `tco_ctx` from parent
- `Let`/`LetRec` body: propagate `tco_ctx` from parent
- `Let`/`LetRec` value: pass `None` (not in tail position)
- `App` arguments: pass `None` (not in tail position)
- All other expressions: pass `None`

### 3. Self-Tail-Call Detection in `App`

In the `App` handler, when `tco_ctx` is `Some`:
1. `flatten_apps()` to get `(func_name, args)`
2. If `func_name == tco_ctx.self_name`:
   a. Compile all args with `tco_ctx = None`
   b. `builder.def_var(tco_ctx.params[i], arg_values[i])` for each param
   c. `builder.ins().jump(tco_ctx.entry_block, &[])`
   d. Create a new block after the jump, switch to it (Cranelift requires)
   e. Return a dummy value (unreachable but needed for type)

### 4. Entry Block Setup

In `define_function()` (or wherever functions are compiled):
- Record the entry block before compiling the body
- Create `TcoContext` with the function's name, entry block, and param Variables
- Pass it to `compile_expr()` for the body

### 5. Parameter Variables

Currently params are declared as Cranelift Variables in function compilation. Store these Variable IDs in `TcoContext.params` so the tail call can reassign them before jumping.

## What NOT to Change

- `compile_expr` return type stays `Result<Value>`
- Core IR — no changes
- Runtime FFI — no changes
- Non-recursive calls — unaffected
- Closure calls — unaffected (indirect calls never match self_name)

## Risk

- Cranelift requires that after a `jump`, the current block is "filled" and you must switch to a new block. The new block is unreachable but satisfies Cranelift's invariant.
- Argument evaluation order: must compile all args BEFORE reassigning any params (avoid clobbering).
