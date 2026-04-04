# Tasks: env-args

## Checklist

- [x] T1: Add `args: Vec<String>` field to `Evaluator` struct in `eval.rs`, add `with_args` constructor
- [x] T2: In `builtin_env()`, inject `args` as `Value::List(Vec<Value::Str>)` at top-level
- [x] T3: Add `env` (arity 1) and `env_or` (arity 2) to the builtins list in `builtin_env()`
- [x] T4: Implement `env` and `env_or` in `call_builtin` using `std::env::var`
- [x] T5: Add `eval_main_with_args(source, base_dir, args)` to `lib.rs` — spawns 64MB thread with args
- [x] T6: In `main.rs`, parse `--` separator from `positional` to extract `script_args`
- [x] T7: Update `run_file` to accept and forward `script_args` to `eval_main_with_args`
- [x] T8: Tests — `env "HOME"` returns non-empty; `env "NONEXISTENT_VAR_SYNOEMA_12345"` returns `""`
- [x] T9: Tests — `env_or "NONEXISTENT" "default"` returns `"default"`
- [x] T10: Tests — `args` is empty list by default when called with no `--`
- [x] T11: Tests — evaluator.with_args(["a","b","c"]) injects `args = ["a" "b" "c"]` into env
- [x] T12: `cargo test` — 0 failures, 0 warnings
