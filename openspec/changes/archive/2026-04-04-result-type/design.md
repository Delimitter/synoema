# Design: result-type

## Architecture

### 1. Prelude Loading — in `lib.rs` / `eval_main_inner()`

The prelude is loaded at the library level, before user code evaluation. This ensures both `eval_main` and `eval_expr` benefit from prelude.

```
prelude.sno → parse → resolve_imports → typecheck → eval_program → Env with Result/Ok/Err/combinators
user.sno → parse → resolve_imports → typecheck → eval_program(env_with_prelude) → final Value
```

**Key decision:** Prelude is prepended to user source before parsing, not loaded as a separate program. This is simpler and avoids dual-environment merging.

Strategy: concatenate prelude source + user source → single parse → single typecheck → single eval. This ensures:
- Type unification between prelude types and user code works naturally
- No env-merging complexity
- Constructors registered in one pass

```rust
const PRELUDE: &str = include_str!("../../prelude/prelude.sno");

fn prepend_prelude(user_source: &str) -> String {
    format!("{}\n{}", PRELUDE, user_source)
}
```

Called in `eval_main_inner()` before `parse()`.

### 2. `error` builtin — Interpreter

Added to `builtin_env()` as 1-arity builtin. In `call_builtin`:

```rust
"error" => {
    let msg = match &args[0] {
        Value::Str(s) => s.clone(),
        v => format!("{}", v),
    };
    Err(EvalError { kind: EvalErrorKind::Type, message: msg })
}
```

### 3. `error` builtin — JIT

In `runtime.rs`:
```rust
pub extern "C" fn synoema_error(msg_ptr: i64) -> i64 {
    let s = tagged_to_str(msg_ptr);
    panic!("error: {}", s);
}
```

Registered in `compiler.rs` `declare_runtime_functions()` with signature `(i64) -> i64`.

### 4. Prelude Content — minimal

Only Result type and combinators. No Map, no JSON, no other types. Keeps prelude small and fast to parse.

### 5. Prelude and JIT

For JIT path: prelude is also prepended to source. The JIT compiler processes the combined program, so Result constructors get ctor_tags and combinators get compiled as normal functions. No special JIT handling needed.

## Files Changed

| File | Change |
|------|--------|
| `lang/prelude/prelude.sno` | New — Result ADT + 7 combinators |
| `lang/crates/synoema-eval/src/lib.rs` | Prepend prelude to user source |
| `lang/crates/synoema-eval/src/eval.rs` | Add `error` builtin |
| `lang/crates/synoema-codegen/src/runtime.rs` | Add `synoema_error` FFI |
| `lang/crates/synoema-codegen/src/compiler.rs` | Register `synoema_error` |
| `CLAUDE.md` | Update status |
| `docs/llm/stdlib.md` | Add Result combinators |
| `context/PROJECT_STATE.md` | Add prelude + Result to status |

## Test Strategy

1. Unit tests in eval crate — Result combinators
2. Unit tests for `error` builtin
3. Integration: pipe chains with Result
4. Regression: all existing 864+ tests must pass
5. JIT tests for `error` builtin
