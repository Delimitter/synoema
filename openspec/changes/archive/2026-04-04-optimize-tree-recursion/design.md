# Design: Tree Recursion Linearization

## Pattern Detection

Detect definitions matching this Core IR shape:

```
CoreDef {
  name: F,
  body: Lam(x, Case(Var(x), [
    Alt(Lit(Int(base0)), val0),
    Alt(Lit(Int(base1)), val1),
    Alt(Var(n), App(App(PrimOp(OP), App(Var(F), sub(n, step1))), App(Var(F), sub(n, step2))))
  ]))
}
```

Where:
- `base0 < base1` (ordered base cases, typically 0 and 1)
- `step1 < step2` (e.g., 1 and 2)
- `OP` is an integer-arithmetic PrimOp (Add, Mul)
- Both recursive calls reference the same function `F` (self-recursion)
- `sub(n, k)` = `App(App(PrimOp(Sub), Var(n)), Lit(Int(k)))`

## Transformation

Generate two definitions replacing the original:

### Worker function (tail-recursive)
```
CoreDef {
  name: "{F}__worker",
  body: Lam(x, Lam(a, Lam(b,
    Case(Var(x), [
      Alt(Lit(Int(0)), Var(a)),
      Alt(Var(_), App(App(App(Var("{F}__worker"),
        App(App(PrimOp(Sub), Var(x)), Lit(Int(1)))),
        Var(b)),
        App(App(PrimOp(OP), Var(a)), Var(b))))
    ])
  )))
}
```

### Wrapper function
```
CoreDef {
  name: F,
  body: Lam(x, App(App(App(Var("{F}__worker"),
    App(App(PrimOp(Sub), Var(x)), Lit(Int(base0)))),
    val0),
    val1))
}
```

If base0 == 0, the wrapper simplifies to just `App(App(App(Var(worker), Var(x)), val0), val1)`.

## Integration

Add `linearize_tree_recursion(program: CoreProgram) -> CoreProgram` as a program-level pass.
Call it from `optimize_program` before `fold_expr` (so constant folding can simplify the result).

## Correctness Argument

For f(n) = f(n-1) OP f(n-2) with f(0)=v0, f(1)=v1:

Worker invariant: `worker(k, f(n-k-1), f(n-k))` = `f(n)` for 0 <= k <= n.
- Base: `worker(0, f(n-1), f(n))` — but we compute forward, not backward.

Actually, the correct invariant for the forward computation:
- `worker(k, f(k-1), f(k))` computes by iterating forward
- `f(n) = wrapper(n) = worker(n, v0, v1)` where n is adjusted for base

Verified by induction for Fibonacci: fib(n) = worker(n, 0, 1).

## Conservative Guards

Only apply the transformation when ALL conditions are met:
1. Exactly 3 case alternatives (two literal base cases + one recursive)
2. Base cases are consecutive integers (e.g., 0,1)
3. Recursive case has exactly 2 self-recursive calls
4. Recursive calls use Sub with literal offsets
5. Calls are combined with a single PrimOp (Add or Mul)
6. No other references to the function name in the recursive case

If any condition fails, leave the definition unchanged.
