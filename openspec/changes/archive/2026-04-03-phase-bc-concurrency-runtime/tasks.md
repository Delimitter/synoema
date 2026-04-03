# Tasks: Phase B + C — Structured Concurrency + Typed Channels

## Checklist

### Phase B: scope { spawn expr }

- [ ] B1: Лексер — токены KwScope и KwSpawn
- [ ] B2: AST — ExprKind::Scope(Box<Expr>) и ExprKind::Spawn(Box<Expr>)
- [ ] B3: Парсер — парсинг `scope { expr }` и `spawn expr`
- [ ] B4: Type checker — правила T-Scope и T-Spawn; check_linear_in_expr
- [ ] B5: Core IR — CoreExpr::Spawn, CoreExpr::Scope (sequential JIT stub)
- [ ] B6: Desugarer — обработка Spawn/Scope в desugar_expr
- [ ] B7: JIT compiler — compile Spawn (sequential), compile Scope (body only)
- [ ] B8: Evaluator — SCOPE_STACK thread-local + eval Spawn/Scope
- [ ] B9: Тесты scope/spawn (≥8)

### Phase C: chan / send / recv

- [ ] C1: Value::Chan + ChanInner struct в value.rs
- [ ] C2: Тип Chan a в infer.rs + chan/send/recv в builtin_env
- [ ] C3: Evaluator builtins: chan, send, recv в call_builtin
- [ ] C4: Runtime FFI: synoema_chan_new, synoema_chan_send, synoema_chan_recv
- [ ] C5: JIT compiler: compile chan/send/recv как FFI calls
- [ ] C6: Тесты chan/send/recv (≥8)

### Final

- [ ] Z1: cargo test — 0 failures, 0 warnings

---

## Детализация

### B1: Лексер — токены
Файлы: `lang/crates/synoema-lexer/src/token.rs`, `scanner.rs`

**token.rs:**
- Добавить `KwScope` и `KwSpawn` в `enum Token` после `KwLazy`
- Добавить в `describe()`: `Token::KwScope => "'scope'"`, `Token::KwSpawn => "'spawn'"`
- Добавить в `is_keyword()`: `Token::KwScope | Token::KwSpawn`
- Обновить `Display` impl (уже обрабатывается через `describe()` в `_` arm)

**scanner.rs:**
- Найти место где сканируются ключевые слова (`"mod"`, `"use"`, `"trait"`, `"impl"`, `"true"`, `"false"`, `"lazy"`)
- Добавить `"scope" => Token::KwScope` и `"spawn" => Token::KwSpawn`

### B2: AST — новые ExprKind
Файл: `lang/crates/synoema-parser/src/ast.rs`

Добавить в `ExprKind` enum:
```rust
/// Structured concurrency scope: `scope { body }`
/// All threads spawned within body are joined before scope returns.
Scope(Box<Expr>),

/// Spawn expression: `spawn expr`
/// Runs expr in a new OS thread within the nearest enclosing scope.
/// Returns Unit.
Spawn(Box<Expr>),
```

### B3: Парсер
Файл: `lang/crates/synoema-parser/src/parser.rs`

В `parse_atom()` (или аналогичной функции для prefix expressions):
```
Token::KwScope => {
    // consume '{'
    expect(Token::LBrace)?;
    let body = parse_expr()?;
    expect(Token::RBrace)?;
    Ok(Expr::new(ExprKind::Scope(Box::new(body)), span))
}
Token::KwSpawn => {
    let expr = parse_app_expr()?; // parse one application
    Ok(Expr::new(ExprKind::Spawn(Box::new(expr)), span))
}
```

Spawn парсит application-level expression (не полное выражение, чтобы избежать захвата `;`).
Пример: `spawn f x` = `Spawn(App(f, x))`, `spawn (f x ; g y)` = `Spawn(Seq(App(f,x), App(g,y)))`.

### B4: Type checker
Файл: `lang/crates/synoema-types/src/infer.rs`

В `infer_inner`:
```rust
ExprKind::Spawn(e) => {
    let (s1, _ty) = self.infer(env, e)?;
    Ok((s1, Type::unit()))
}
ExprKind::Scope(body) => {
    self.infer(env, body)
}
```

В `check_linear_in_expr`:
```rust
ExprKind::Spawn(e) | ExprKind::Scope(e) => check_linear_in_expr(e, linear_vars),
```

### B5: Core IR
Файл: `lang/crates/synoema-core/src/core_ir.rs`

Добавить в `CoreExpr`:
```rust
/// spawn expr — Phase B: sequential stub; Phase C: real OS thread
Spawn(Box<CoreExpr>),
/// scope { body } — Phase B: sequential (no thread mgmt in JIT)
Scope(Box<CoreExpr>),
```

Обновить `Display` impl для новых вариантов (добавить match arms).
Обновить `optimize` в `optimize.rs` (pass-through, как для Case).

### B6: Desugarer
Файл: `lang/crates/synoema-core/src/desugar.rs`

В `desugar_expr`:
```rust
ExprKind::Spawn(e) => CoreExpr::Spawn(Box::new(desugar_expr(fresh, e))),
ExprKind::Scope(body) => CoreExpr::Scope(Box::new(desugar_expr(fresh, body))),
```

### B7: JIT compiler (Phase B stub)
Файл: `lang/crates/synoema-codegen/src/compiler.rs`

В `compile_expr`:
```rust
CoreExpr::Spawn(e) => {
    // Phase B: sequential stub — evaluate but discard result
    let val = self.compile_expr(builder, func_ctx, e)?;
    // result is Unit (0) — spawn runs synchronously, no thread
    Ok(val)
}
CoreExpr::Scope(body) => {
    // Phase B: sequential stub — evaluate body directly
    self.compile_expr(builder, func_ctx, body)
}
```

Phase C will replace these with real FFI calls.

### B8: Evaluator — SCOPE_STACK + eval Spawn/Scope
Файл: `lang/crates/synoema-eval/src/eval.rs`

Добавить thread-local в начало файла:
```rust
thread_local! {
    static SCOPE_STACK: RefCell<Vec<Vec<std::thread::JoinHandle<()>>>> = RefCell::new(Vec::new());
}
```

В `eval()` match:
```rust
ExprKind::Spawn(expr) => {
    let env_clone = env.clone();
    let expr_clone = expr.as_ref().clone();
    let handle = std::thread::spawn(move || {
        let mut ev = Evaluator::new();
        let _ = ev.eval(&env_clone, &expr_clone);
    });
    SCOPE_STACK.with(|s| {
        if let Some(top) = s.borrow_mut().last_mut() {
            top.push(handle);
        }
        // spawn outside scope: thread runs detached
    });
    Ok(Value::Unit)
}

ExprKind::Scope(body) => {
    SCOPE_STACK.with(|s| s.borrow_mut().push(Vec::new()));
    let result = self.eval(env, body);
    let handles = SCOPE_STACK.with(|s| s.borrow_mut().pop().unwrap_or_default());
    for h in handles { let _ = h.join(); }
    result
}
```

### B9: Тесты scope/spawn
Файл: `lang/crates/synoema-eval/src/tests.rs`

- `scope_basic` — `scope { spawn (print "hello") }` не паникует, возвращает Unit
- `scope_returns_result` — `scope { 42 }` возвращает 42
- `scope_waits_for_spawn` — spawn изменяет shared state (через channel) до scope exit
- `scope_multiple_spawns` — несколько spawn в одном scope
- `scope_nested` — вложенные scope работают корректно
- `spawn_outside_scope` — spawn без scope не паникует (no-op или detached)
- `scope_sequential_semantics` — scope { e1 ; e2 } вычисляет в правильном порядке
- `scope_with_seq` — `scope { spawn f ; spawn g }` — оба запускаются

### C1: Value::Chan + ChanInner
Файл: `lang/crates/synoema-eval/src/value.rs`

```rust
pub struct ChanInner {
    pub sender:   std::sync::Mutex<std::sync::mpsc::Sender<Value>>,
    pub receiver: std::sync::Mutex<std::sync::mpsc::Receiver<Value>>,
}

// В Value enum:
Chan(std::sync::Arc<ChanInner>),
```

Обновить `PartialEq` (Chan == Chan всегда false, или по ptr equality).
Обновить `Display`: `Chan(_) => write!(f, "<chan>")`.

### C2: Chan type + builtins в type checker
Файл: `lang/crates/synoema-types/src/infer.rs`

Добавить helper в `types.rs`:
```rust
pub fn chan(elem: Type) -> Self {
    Type::App(Box::new(Type::Con("Chan".into())), Box::new(elem))
}
```

В `initial_env()` в `infer.rs`:
```rust
// chan: ∀a. Chan a  (0-arity — returns fresh channel)
let ca = self.gen.fresh();
env.insert("chan".into(), Scheme { vars: vec![ca], ty: Type::chan(Type::Var(ca)) });

// send: ∀a. Chan a -> a -> Unit
let sa = self.gen.fresh();
env.insert("send".into(), Scheme {
    vars: vec![sa],
    ty: Type::arrow(Type::chan(Type::Var(sa)), Type::arrow(Type::Var(sa), Type::unit())),
});

// recv: ∀a. Chan a -> a
let ra = self.gen.fresh();
env.insert("recv".into(), Scheme {
    vars: vec![ra],
    ty: Type::arrow(Type::chan(Type::Var(ra)), Type::Var(ra)),
});
```

### C3: Evaluator builtins
Файл: `lang/crates/synoema-eval/src/eval.rs`

В `builtin_env()`:
```rust
env.insert("chan".to_string(), Value::Builtin("chan".to_string(), 0));
// send: arity 2, recv: arity 1
for (name, arity) in &[("send", 2), ("recv", 1)] {
    env.insert(name.to_string(), Value::Builtin(name.to_string(), *arity));
}
```

В `call_builtin`:
```rust
"chan" => {
    let (tx, rx) = std::sync::mpsc::channel::<Value>();
    Ok(Value::Chan(std::sync::Arc::new(ChanInner {
        sender: std::sync::Mutex::new(tx),
        receiver: std::sync::Mutex::new(rx),
    })))
}
"send" => match &args[0] {
    Value::Chan(c) => {
        c.sender.lock().map_err(|_| err("send: poisoned"))?
            .send(args[1].clone())
            .map_err(|_| err("send: channel closed"))?;
        Ok(Value::Unit)
    }
    _ => Err(err("send: expected Chan")),
},
"recv" => match &args[0] {
    Value::Chan(c) => c.receiver.lock()
        .map_err(|_| err("recv: poisoned"))?
        .recv()
        .map_err(|_| err("recv: channel closed")),
    _ => Err(err("recv: expected Chan")),
},
```

### C4: Runtime FFI
Файл: `lang/crates/synoema-codegen/src/runtime.rs`

Определить `ChanNode` (heap-allocated канал):
```rust
#[repr(C)]
pub struct ChanNode {
    inner: *mut std::sync::Arc<ChanInnerRt>,
}

pub struct ChanInnerRt {
    pub sender: std::sync::Mutex<std::sync::mpsc::Sender<i64>>,
    pub receiver: std::sync::Mutex<std::sync::mpsc::Receiver<i64>>,
}
```

Экспортировать функции:
```rust
#[unsafe(no_mangle)]
pub extern "C" fn synoema_chan_new() -> i64 { ... }

#[unsafe(no_mangle)]
pub extern "C" fn synoema_chan_send(chan: i64, val: i64) -> i64 { ... }

#[unsafe(no_mangle)]
pub extern "C" fn synoema_chan_recv(chan: i64) -> i64 { ... }
```

Tag для ChanNode: определить на основе текущей схемы тегов в runtime.rs (проверить CHAN_TAG).

### C5: JIT compiler — chan/send/recv
Файл: `lang/crates/synoema-codegen/src/compiler.rs`

После изучения текущей структуры (как компилируются print, show, tcp_listen), добавить:
- Объявить `synoema_chan_new`, `synoema_chan_send`, `synoema_chan_recv` в `declare_runtime_functions()`
- В `compile_expr` для `CoreExpr::App(Var("chan"), ...)` и встроенных вызовов: emit FFI call

Или проще: добавить в `compile_app_builtins` (если такая функция есть):
```rust
"chan" => self.emit_call_0(builder, "synoema_chan_new"),
"send" => self.emit_call_2(builder, "synoema_chan_send", ch_val, val_val),
"recv" => self.emit_call_1(builder, "synoema_chan_recv", ch_val),
```

Детали зависят от архитектуры compiler.rs (изучить при реализации).

### C6: Тесты chan/send/recv
Файл: `lang/crates/synoema-eval/src/tests.rs` + `lang/crates/synoema-codegen/src/lib.rs`

Interpreter tests:
- `chan_create` — `chan` создаёт Value::Chan
- `chan_send_recv` — send 42, recv возвращает 42
- `chan_in_scope` — producer/consumer через scope+spawn
- `chan_multiple_messages` — send/recv несколько значений подряд
- `chan_string` — каналы для строк (проверка типовой полиморфности в runtime)
- `chan_type_check` — type checker принимает корректный код
- `chan_type_error` — type checker отклоняет send ch wrong_type
- `chan_concurrent` — реальная конкурентность: producer в spawn, consumer в main

### Z1: cargo test
```bash
cd lang && cargo test
```
0 failures, 0 warnings.
