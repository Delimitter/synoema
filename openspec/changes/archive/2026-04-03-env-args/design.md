# Design: env-args

## Technical Decisions

### 1. `env` and `env_or` as builtins

Add to `builtin_env()` in `eval.rs`:

```rust
("env", 1), ("env_or", 2),
```

Implement in `call_builtin`:

```rust
"env" => match &args[0] {
    Value::Str(name) => Ok(Value::Str(std::env::var(name).unwrap_or_default())),
    _ => Err(...),
},
"env_or" => match (&args[0], &args[1]) {
    (Value::Str(name), Value::Str(default)) =>
        Ok(Value::Str(std::env::var(name).unwrap_or_else(|_| default.clone()))),
    _ => Err(...),
},
```

### 2. `args` injection

**Approach:** Add `args: Vec<String>` field to `Evaluator`. Inject into `builtin_env()` as `Value::List(...)`. Add `Evaluator::with_args(args: Vec<String>) -> Self` constructor.

Add new public API to `lib.rs`:

```rust
pub fn eval_main_with_args(
    source: &str,
    base_dir: Option<&Path>,
    args: Vec<String>,
) -> Result<(Value, Vec<String>), Diagnostic>
```

This spawns the 64MB thread with `args` passed through.

**Why not thread-local/global:** Cleaner, testable, no hidden state.

### 3. `--` parsing in main.rs

In `main.rs`, after `positional` is built, detect `--` position:

```rust
let dash_dash_pos = positional.iter().position(|a| *a == "--");
let script_args: Vec<String> = dash_dash_pos
    .map(|i| positional[i+1..].iter().map(|s| s.to_string()).collect())
    .unwrap_or_default();
let positional = dash_dash_pos
    .map(|i| &positional[..i])
    .unwrap_or(&positional);
```

Pass `script_args` to `eval_main_with_args` in `run_file`.

### 4. `run_file` signature update

```rust
fn run_file(path: &str, format: ErrorFormat, script_args: Vec<String>)
```

`jit_file` does NOT receive args (JIT is out of scope).

## Files Changed

| File | Change |
|------|--------|
| `eval/src/eval.rs` | +`args` field on `Evaluator`, +`with_args`, +`env`/`env_or` builtins |
| `eval/src/lib.rs` | +`eval_main_with_args` public API |
| `repl/src/main.rs` | Parse `--` separator, pass `script_args` to `run_file` |
| `eval/src/tests.rs` | +6 tests |

## BPE Verification

- `env` = BPE token 549 (1 token) ✓
- `env_or` = "env" + "_" + "or" — 3 tokens. Must verify.
  - Alternative: `envOr` or use `env_or` and document as multi-token acceptable
  - Check: `tools/bpe-verify/verify_bpe.py`
- `args` = BPE token (1 token) ✓

> Note: `env_or` may be 2 BPE tokens ("_or" appended). If so, consider `envOr` (still 2: "env"+"Or"). Since these are builtin function names (not operators), the 1-token rule applies to syntax operators. Function names can be multi-token — BPE rule is for operators/keywords only. Document this explicitly.
