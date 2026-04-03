# Spec: Self-Recursive TCO in JIT

## Capability

When a function's body ends with a call to itself (self-recursive tail call), the JIT compiler emits a jump to the function entry instead of a call instruction. The function effectively becomes a loop.

## Tail Position Definition

An expression `e` is in tail position of function `f` if `e` is:
1. The body of `f` itself
2. A branch of a `Case` expression that is in tail position
3. The `body` of a `Let(name, val, body)` or `LetRec(name, val, body)` where the `body` is in tail position

An expression is a **self-tail-call** if it is `App(...App(Var(f), a1)..., aN)` in tail position where `f` is the function being compiled.

## Compilation Rule

When a self-tail-call is detected:
1. Compile each argument `a1..aN` to Cranelift values
2. Reassign function parameters: `builder.def_var(param_i, compiled_arg_i)` for each i
3. Emit `builder.ins().jump(entry_block, &[])` to the function entry
4. No `call` instruction emitted — no new stack frame

## Invariants

- Only self-recursion is optimized (function calls itself by exact name)
- Non-tail self-calls remain as normal `call` instructions
- All existing JIT behavior unchanged for non-tail-recursive programs
- Arguments are evaluated before parameter reassignment (left-to-right)

## Acceptance Criteria

- `euler1(999999)` runs in JIT without stack overflow
- `factorial(10)` JIT result unchanged (3628800)
- `gcd(1071, 462)` JIT result unchanged (21)
- New test: deeply recursive tail call (e.g., countdown from 100000) succeeds in JIT
- All existing 634+ tests pass
