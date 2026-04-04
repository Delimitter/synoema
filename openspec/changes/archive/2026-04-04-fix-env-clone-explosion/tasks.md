# Tasks: Fix Env Clone Explosion

## T-1: Value types — Arc<Env>
- [x] Change `Value::Closure { env: Env }` → `Value::Closure { env: Arc<Env> }` in value.rs
- [x] Change `Value::Func { env: Env }` → `Value::Func { env: Arc<Env> }` in value.rs
- [x] Arc already imported in value.rs (used by ChanInner)
- [x] `Env::child()` works unchanged via Deref (Arc<Env> → &Env → child())

## T-2: eval_program — fix clone explosion
- [x] Pass 2: register functions with `Arc::new(env.clone())` (temporary)
- [x] Pass 3: clone env (now O(N) not O(N!)), create snapshot, update functions with `Arc::new(snapshot.clone())`
- [x] Mutual recursion works: snapshot contains all function names, each function's env = snapshot

## T-3: eval.rs — update all env call sites
- [x] Line 245 (lambda): `env: Arc::new(env.clone())`
- [x] apply() Closure multi-param: `env: Arc::new(local)`
- [x] apply() Func Case 1 (0-pattern): `env: Arc::clone(&env)`
- [x] apply() Func Case 2 (curry): `env: Arc::clone(&env)` for self-ref, `env: Arc::new(local)` for result
- [x] apply() Func Case 3 (single-pat): `env: Arc::clone(&env)` for self-ref, `env: Arc::new(local)` for result
- [x] Spawn path (line 405): unchanged — `env.clone()` works via &Env, Value is Send (Arc is Send+Sync)

## T-4: Fix Box::leak in JIT string handling
- [x] runtime.rs synoema_readline: use `line.as_bytes()` directly (synoema_str_new copies to arena)
- [x] compiler.rs string literal: use `s.as_ptr()` directly (AST outlives JIT execution)

## T-5: Tests and verification
- [x] `cargo test` — 0 failures, 0 warnings (only pre-existing warning about unused Write import)
- [x] `cargo run -p synoema-repl -- eval "6 * 7"` → 42, <1s, normal memory
- [x] All 875+ tests pass (217+49+60+29+238+17+46+51+10+88+21+90+9 = 925 run, 20 ignored)
