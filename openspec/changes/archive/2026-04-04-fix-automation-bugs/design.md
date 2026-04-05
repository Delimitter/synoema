# Design: Automation Bug Fixes

## Fix 1: Pattern Matching (eval.rs)

**Current code** (lines 513-526): Loop over equations, try bind each against first arg, merge all probe bindings into shared `local`:
```rust
for eq in &equations {
    let mut probe = local.child();
    if self.try_bind_pattern(&eq.pats[0], &arg, &mut probe) {
        for (k, v) in probe.bindings() {
            local.insert(k.clone(), v.clone()); // ← overwrites
        }
        remaining.push(...);
    }
}
```

**Fix**: Don't merge bindings into shared local. Instead, store per-equation bindings in the remaining closure environment. Each remaining equation gets its OWN environment snapshot with only its own bindings.

**Approach**: Instead of merging to `local`, create a per-equation child env that contains only that equation's bindings, and store it alongside the remaining equation.

## Fix 2: Type Checker (infer.rs)

Add two lines to `builtin_env()` after the `even` registration (~line 440):
```rust
env.insert("not".into(), Scheme::mono(Type::arrow(Type::bool(), Type::bool())));
env.insert("odd".into(), Scheme::mono(Type::arrow(Type::int(), Type::bool())));
```

## Fix 3: Parser (parser.rs)

Add `Token::LBracket` to the constructor argument loop at ~line 572:
```rust
Token::LBracket => {
    args.push(self.parse_pattern()?);
}
```

This delegates to `parse_pattern()` which already handles list patterns.
