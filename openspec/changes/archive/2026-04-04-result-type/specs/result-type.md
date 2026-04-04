# Spec: result-type

## Prelude Mechanism

**File:** `lang/prelude/prelude.sno`
**Embedding:** `include_str!("../../prelude/prelude.sno")` in eval crate
**Loading:** Before user declarations in `eval_program()`
**Visibility:** All prelude definitions available in user scope without import

### Prelude Loading Contract
1. Prelude source is parsed via `synoema_parser::parse()`
2. Prelude imports are resolved (for future extensibility, currently none)
3. Prelude is typechecked via `synoema_types::typecheck_program()`
4. Prelude declarations are evaluated into the environment
5. User program is then evaluated in this enriched environment
6. Prelude errors are internal panics (not user-facing diagnostics)

## Result ADT

```
Result a e = Ok a | Err e
```

### Combinators

| Function | Type | Behaviour |
|----------|------|-----------|
| `map_ok` | `(a -> b) -> Result a e -> Result b e` | Apply f to Ok value |
| `map_err` | `(e -> f) -> Result a e -> Result a f` | Apply f to Err value |
| `unwrap` | `Result a e -> a` | Extract Ok, `error` on Err |
| `unwrap_or` | `a -> Result a e -> a` | Extract Ok or use default |
| `is_ok` | `Result a e -> Bool` | True if Ok |
| `is_err` | `Result a e -> Bool` | True if Err |
| `and_then` | `(a -> Result b e) -> Result a e -> Result b e` | Monadic bind |

### Function Definitions (Synoema syntax)

```
map_ok f (Ok x) = Ok (f x)
map_ok f (Err e) = Err e

map_err f (Ok x) = Ok x
map_err f (Err e) = Err (f e)

unwrap (Ok x) = x
unwrap (Err e) = error ("unwrap called on Err: " ++ show e)

unwrap_or def (Ok x) = x
unwrap_or def (Err _) = def

is_ok (Ok _) = true
is_ok (Err _) = false

is_err (Ok _) = false
is_err (Err _) = true

and_then f (Ok x) = f x
and_then f (Err e) = Err e
```

## `error` Builtin

### Interpreter
- Registered in `builtin_env()` with arity 1
- `error msg` → `EvalError { kind: Type, message: msg }`

### JIT
- `extern "C" fn synoema_error(msg_ptr: i64) -> i64`
- Extracts string from tagged pointer, panics with message
- Declared in `compiler.rs` runtime function registry
- Returns `i64` in signature (never actually returns — panics)

## No New Dependencies

All changes use existing infrastructure: parser, typechecker, evaluator, runtime FFI.
