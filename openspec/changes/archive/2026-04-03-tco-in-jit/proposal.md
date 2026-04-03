# TCO in JIT

## Problem

Interpreter handles deep recursion via 64MB stack thread (Phase 10.1), but JIT compiles every recursive call as a standard `call` instruction — each call pushes a new stack frame. For programs like `euler1(999999)` through JIT, this causes stack overflow.

## Current State

- **Interpreter:** `eval_main()` spawns thread with 64MB stack. No true TCO — just a large stack.
- **JIT:** Recursive calls compiled as direct `call` instructions via `flatten_apps()` → `builder.ins().call()`. No tail call detection or optimization.
- **Core IR:** No tail position annotation (`CoreExpr` has `App` but no `TailApp` variant).

## Proposed Solution

Implement self-recursive TCO in the JIT compiler by converting tail-position self-calls into jumps back to the function entry block. This is the standard "tail recursion → loop" transformation.

### Scope

- **Self-recursive tail calls only** (not mutual recursion or tail calls to other functions)
- **Direct calls only** (not indirect/closure calls)
- Transform at JIT compilation level — no Core IR changes needed

### Approach

1. In `compile_function()` / `define_function()`, track the current function name and its entry block
2. When compiling a function body, detect if the final expression is `App(App(...(Var(self_name), arg1), arg2), ...argN)` in tail position
3. Instead of emitting `call`, emit: reassign parameters → `jump` to entry block
4. Tail position = the expression whose value is returned from the function (last expression in body, then-branch/else-branch of case in tail position, body of let in tail position)

### Not in Scope

- Mutual recursion TCO
- General tail calls (to arbitrary functions)
- Core IR annotation pass
- Cranelift `return_call` instruction (adds complexity, self-jump is simpler)

## Impact

- `euler1(999999)` in JIT: stack overflow → works
- All existing recursive JIT tests: same behavior (non-tail calls unchanged)
- Performance: tail-recursive functions become loops — faster + constant stack
