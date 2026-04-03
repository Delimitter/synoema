# Delta Spec: Concurrency Primitives (Phase B + C)

## 1. Lexer Changes

### New Keywords (Token variants)
```rust
Token::KwScope  // 'scope'
Token::KwSpawn  // 'spawn'
```

Both are 1 BPE-token in cl100k_base. Scanner recognizes them as keyword identifiers.

`chan`, `send`, `recv` are NOT keywords — they are builtin-function names in the initial environment.

## 2. AST Changes

### New ExprKind variants
```rust
/// Structured concurrency scope: `scope { body }`
/// Launches body, joins all spawned threads, returns body result.
Scope(Box<Expr>),

/// Spawn expression: `spawn expr`
/// Evaluates expr in a new OS thread (within nearest enclosing scope). Returns Unit.
Spawn(Box<Expr>),
```

### Parser grammar
```
atom ::= ... | 'scope' '{' expr '}' | 'spawn' app_expr
```

- `scope { expr }` — consumes `{`, parses single expression, consumes `}`
- `spawn expr` — parses a single application-level expression (right-hand atom)
- Precedence: `spawn` binds tighter than `;`, same as function application

## 3. Type System

### New type constructor
`Chan` — builtin type constructor, arity 1.
`Chan a` = `Type::App(Type::Con("Chan"), Type::Var(a))`

Helper: `Type::chan(elem) = Type::App(Box::new(Type::Con("Chan".into())), Box::new(elem))`

### Typing rules

```
Γ ⊢ e : a
──────────────────────  [T-Spawn]
Γ ⊢ spawn e : Unit

Γ ⊢ body : a
──────────────────────  [T-Scope]
Γ ⊢ scope { body } : a
```

### Builtin type signatures
```
chan  : ∀a. Chan a
send : ∀a. Chan a → a → Unit
recv : ∀a. Chan a → a
```

Added to the initial type environment in `infer.rs`:
```rust
// chan: ∀a. Chan a
let ca = self.gen.fresh();
env.insert("chan", Scheme { vars: vec![ca], ty: Type::chan(Type::Var(ca)) });

// send: ∀a. Chan a → a → Unit
let sa = self.gen.fresh();
env.insert("send", Scheme {
    vars: vec![sa],
    ty: Type::arrow(Type::chan(Type::Var(sa)), Type::arrow(Type::Var(sa), Type::unit())),
});

// recv: ∀a. Chan a → a
let ra = self.gen.fresh();
env.insert("recv", Scheme {
    vars: vec![ra],
    ty: Type::arrow(Type::chan(Type::Var(ra)), Type::Var(ra)),
});
```

## 4. Runtime Values

```rust
// In value.rs:
pub struct ChanInner {
    pub sender:   std::sync::Mutex<std::sync::mpsc::Sender<Value>>,
    pub receiver: std::sync::Mutex<std::sync::mpsc::Receiver<Value>>,
}

// New Value variant:
Chan(std::sync::Arc<ChanInner>),
```

`Value::Chan` is `Clone` via Arc. `Value: Send` (no Rc/RefCell anywhere in Value).

## 5. Interpreter Semantics

### spawn

```rust
// Thread-local scope stack (in eval.rs)
thread_local! {
    static SCOPE_STACK: RefCell<Vec<Vec<JoinHandle<()>>>> = RefCell::new(Vec::new());
}

// eval for Spawn(expr):
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
    // spawn outside scope: handle dropped (thread detached — not ideal but safe)
});
Ok(Value::Unit)
```

### scope

```rust
// eval for Scope(body):
SCOPE_STACK.with(|s| s.borrow_mut().push(Vec::new()));
let result = self.eval(env, body);
let handles = SCOPE_STACK.with(|s| s.borrow_mut().pop().unwrap_or_default());
for h in handles { let _ = h.join(); }
result
```

### chan / send / recv (builtins)

```rust
"chan" => {
    let (tx, rx) = std::sync::mpsc::channel::<Value>();
    Ok(Value::Chan(Arc::new(ChanInner {
        sender: Mutex::new(tx),
        receiver: Mutex::new(rx),
    })))
}
"send" => {
    // args: [Chan, value]
    match &args[0] {
        Value::Chan(c) => {
            c.sender.lock().unwrap().send(args[1].clone())
                .map_err(|_| err("send: channel closed"))?;
            Ok(Value::Unit)
        }
        _ => Err(err("send: expected Chan")),
    }
}
"recv" => {
    match &args[0] {
        Value::Chan(c) => c.receiver.lock().unwrap().recv()
            .map_err(|_| err("recv: channel closed")),
        _ => Err(err("recv: expected Chan")),
    }
}
```

## 6. Core IR Changes (for JIT support in Phase C)

### New CoreExpr variants
```rust
/// Spawn expression (JIT: sequential stub)
Spawn(Box<CoreExpr>),
/// Scope block (JIT: sequential — all spawned are run inline)
Scope(Box<CoreExpr>),
```

### Desugaring
```rust
ExprKind::Spawn(e) => CoreExpr::Spawn(Box::new(desugar_expr(fresh, e))),
ExprKind::Scope(body) => CoreExpr::Scope(Box::new(desugar_expr(fresh, body))),
```

### JIT Codegen (Phase C)

`scope/spawn` in JIT are sequential: `scope { body }` compiles as body, `spawn e` compiles e but wraps in `synoema_scope_spawn(fn_ptr)`.

For chan/send/recv in JIT:
```c
// runtime.rs extern "C":
synoema_chan_new() -> i64            // returns tagged ptr to ChanNode
synoema_chan_send(chan: i64, val: i64) -> i64   // returns 0 (Unit)
synoema_chan_recv(chan: i64) -> i64             // blocking receive
```

`ChanNode` is heap-allocated (arena), stores Arc<ChanInner> as a raw pointer.

## 7. Display / Debug

- `Value::Chan` displays as `<chan>`
- `show (chan)` in type checker should type-check (show is ∀a. a → String)
