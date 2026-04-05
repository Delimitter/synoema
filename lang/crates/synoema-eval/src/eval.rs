// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

//! Tree-walking evaluator for Synoema.
//!
//! Implements big-step operational semantics from the Language Reference §5.
//! Strict (eager) evaluation — arguments are evaluated before substitution.

use crate::value::{Value, Env, ChanInner};
use synoema_parser::*;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

// ── I/O fd registry (per-thread; server is single-threaded) ──────────────────

thread_local! {
    static IO_LISTENERS: RefCell<HashMap<i64, TcpListener>>             = RefCell::new(HashMap::new());
    static IO_READERS:   RefCell<HashMap<i64, Box<dyn BufRead>>>        = RefCell::new(HashMap::new());
    static IO_WRITERS:   RefCell<HashMap<i64, Box<dyn Write>>>          = RefCell::new(HashMap::new());
    static IO_CHILDREN:  RefCell<HashMap<i64, std::process::Child>>     = RefCell::new(HashMap::new());
    static IO_NEXT_FD:   RefCell<i64>                                   = RefCell::new(100);
}

fn next_io_fd() -> i64 {
    IO_NEXT_FD.with(|c| { let mut c = c.borrow_mut(); let fd = *c; *c += 1; fd })
}

// Unique counter for pattern match binding isolation
thread_local! {
    static CURRY_COUNTER: RefCell<u64> = RefCell::new(0);
}

fn next_curry_id() -> u64 {
    CURRY_COUNTER.with(|c| { let mut c = c.borrow_mut(); let id = *c; *c += 1; id })
}

// ── Structured concurrency scope stack ───────────────────────────────────────
// Each scope push adds an empty vec; spawn pushes handles onto the top vec.
// On scope exit the top vec is popped and all handles are joined.
thread_local! {
    static SCOPE_STACK: std::cell::RefCell<Vec<Vec<std::thread::JoinHandle<()>>>>
        = std::cell::RefCell::new(Vec::new());
}

/// Structured error kind — allows precise diagnostic code dispatch without string matching.
#[derive(Debug, Clone, PartialEq)]
pub enum EvalErrorKind {
    /// Variable or constructor not in scope.
    Undefined,
    /// No equation matched the given argument.
    NoMatch,
    /// Division or modulo by zero.
    DivisionByZero,
    /// IO primitive failed (readline, tcp_listen, file_read, …).
    IoError,
    /// Runtime type error (wrong type for an operation).
    Type,
}

/// Evaluation error
#[derive(Debug, Clone)]
pub struct EvalError {
    pub kind: EvalErrorKind,
    pub message: String,
}

impl EvalError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { kind: EvalErrorKind::Type, message: msg.into() }
    }
    pub fn with_kind(kind: EvalErrorKind, msg: impl Into<String>) -> Self {
        Self { kind, message: msg.into() }
    }
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Runtime error: {}", self.message)
    }
}

type EResult<T> = Result<T, EvalError>;

fn err(msg: impl Into<String>) -> EvalError { EvalError::new(msg) }
fn err_undef(msg: impl Into<String>) -> EvalError { EvalError::with_kind(EvalErrorKind::Undefined, msg) }
fn err_no_match(msg: impl Into<String>) -> EvalError { EvalError::with_kind(EvalErrorKind::NoMatch, msg) }
fn err_div_zero(msg: impl Into<String>) -> EvalError { EvalError::with_kind(EvalErrorKind::DivisionByZero, msg) }
fn err_io(msg: impl Into<String>) -> EvalError { EvalError::with_kind(EvalErrorKind::IoError, msg) }

/// The Synoema evaluator
pub struct Evaluator {
    /// Output buffer (for testing — captures print output)
    pub output: Vec<String>,
    /// CLI arguments passed after `--` separator (injected as `args` builtin)
    pub args: Vec<String>,
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator { output: Vec::new(), args: Vec::new() }
    }

    pub fn with_args(args: Vec<String>) -> Self {
        Evaluator { output: Vec::new(), args }
    }

    /// Evaluate a complete program, return the environment
    pub fn eval_program(&mut self, program: &Program) -> EResult<Env> {
        let mut env = self.builtin_env();

        // First pass: register ADT constructors
        for decl in &program.decls {
            if let Decl::TypeDef { variants, .. } = decl {
                for v in variants {
                    let arity = v.fields.len();
                    if arity == 0 {
                        env.insert(v.name.clone(), Value::Con(v.name.clone(), vec![]));
                    } else {
                        env.insert(v.name.clone(), Value::Builtin(format!("ctor:{}", v.name), arity));
                    }
                }
            }
        }

        // Collect impl method equations (more-specific first, to prepend to function equations)
        let mut impl_eqs: std::collections::HashMap<String, Vec<Equation>> =
            std::collections::HashMap::new();
        for decl in &program.decls {
            if let Decl::ImplDecl { methods, .. } = decl {
                for (method_name, equations) in methods {
                    impl_eqs.entry(method_name.clone())
                        .or_default()
                        .extend(equations.iter().cloned());
                }
            }
        }

        // Second pass: register all functions (to enable mutual recursion).
        // Prepend impl equations where applicable.
        for decl in &program.decls {
            if let Decl::Func { name, equations, .. } = decl {
                let prepend = impl_eqs.remove(name).unwrap_or_default();
                let mut all_eqs = prepend;
                all_eqs.extend(equations.iter().cloned());
                let func = Value::Func {
                    name: name.clone(),
                    equations: all_eqs,
                    env: Arc::new(env.clone()),  // temporary, replaced in pass 3
                };
                env.insert(name.clone(), func);
            }
        }

        // Register standalone impl methods not covered by any Func decl
        for (method_name, equations) in impl_eqs {
            let func = Value::Func {
                name: method_name.clone(),
                equations,
                env: Arc::new(env.clone()),  // temporary, replaced in pass 3
            };
            env.insert(method_name, func);
        }

        // Third pass: update function closures to capture the complete environment
        // (enables mutual recursion).
        //
        // Memory fix: with Arc<Env> in Value::Func, env.clone() is O(N) instead
        // of O(N!) — cloning N Values where each clone is Arc::clone = O(1).
        // The old code with owned Env caused 106 GB memory explosion because
        // each snapshot.clone() recursively deep-copied all nested Envs.
        let snapshot = env.clone();
        for decl in &program.decls {
            if let Decl::Func { name, .. } = decl {
                if let Some(Value::Func { equations, .. }) = snapshot.lookup(name) {
                    let equations = equations.clone();
                    let func = Value::Func {
                        name: name.clone(),
                        equations,
                        env: Arc::new(snapshot.clone()),
                    };
                    env.insert(name.clone(), func);
                }
            }
        }
        // Also update standalone impl methods
        for decl in &program.decls {
            if let Decl::ImplDecl { methods, .. } = decl {
                for (method_name, _) in methods {
                    if let Some(Value::Func { equations, .. }) = snapshot.lookup(method_name) {
                        let equations = equations.clone();
                        let func = Value::Func {
                            name: method_name.clone(),
                            equations,
                            env: Arc::new(snapshot.clone()),
                        };
                        env.insert(method_name.clone(), func);
                    }
                }
            }
        }

        Ok(env)
    }

    /// Evaluate an expression in an environment
    pub fn eval(&mut self, env: &Env, expr: &Expr) -> EResult<Value> {
        match &expr.kind {
            // ── Literals ────────────────────────────
            ExprKind::Lit(lit) => Ok(self.eval_lit(lit)),

            // ── Variable ────────────────────────────
            ExprKind::Var(name) => {
                let val = env.lookup(name)
                    .cloned()
                    .ok_or_else(|| err_undef(format!("Undefined variable: {}", name)))?;
                // Force 0-arity functions: `a = 10` is stored as
                // Func { pats: [], body: Lit(10) } — evaluate the body
                match &val {
                    Value::Func { equations, env: fenv, name: fname }
                        if equations.len() == 1 && equations[0].pats.is_empty() =>
                    {
                        let mut local = fenv.child();
                        local.insert(fname.clone(), val.clone());
                        self.eval(&local, &equations[0].body)
                    }
                    // 0-arity builtins (e.g. readline): execute immediately
                    Value::Builtin(ref bname, 0) => {
                        let n = bname.clone();
                        self.call_builtin(&n, &[])
                    }
                    _ => Ok(val),
                }
            }

            // ── Constructor (nullary) ───────────────
            ExprKind::Con(name) => {
                env.lookup(name)
                    .cloned()
                    .ok_or_else(|| err_undef(format!("Undefined constructor: {}", name)))
            }

            // ── Application ─────────────────────────
            ExprKind::App(func_e, arg_e) => {
                let func_v = self.eval(env, func_e)?;
                let arg_v = self.eval(env, arg_e)?;
                self.apply(func_v, arg_v)
            }

            // ── Lambda ──────────────────────────────
            ExprKind::Lam(pats, body) => {
                Ok(Value::Closure {
                    params: pats.clone(),
                    body: *body.clone(),
                    env: Arc::new(env.clone()),
                })
            }

            // ── Binary operator ─────────────────────
            ExprKind::BinOp(op, lhs, rhs) => {
                // Short-circuit for && and ||
                match op {
                    BinOp::And => {
                        let l = self.eval(env, lhs)?;
                        return match l {
                            Value::Bool(false) => Ok(Value::Bool(false)),
                            Value::Bool(true) => self.eval(env, rhs),
                            _ => Err(err("&& requires Bool operands")),
                        };
                    }
                    BinOp::Or => {
                        let l = self.eval(env, lhs)?;
                        return match l {
                            Value::Bool(true) => Ok(Value::Bool(true)),
                            Value::Bool(false) => self.eval(env, rhs),
                            _ => Err(err("|| requires Bool operands")),
                        };
                    }
                    BinOp::Pipe => {
                        // x |> f  =  f x
                        let x = self.eval(env, lhs)?;
                        let f = self.eval(env, rhs)?;
                        return self.apply(f, x);
                    }
                    BinOp::Compose => {
                        // f >> g  =  value that, when applied to x, returns g (f x)
                        let f_val = self.eval(env, lhs)?;
                        let g_val = self.eval(env, rhs)?;
                        // Store f and g in a PartialBuiltin with 1 remaining arg
                        return Ok(Value::PartialBuiltin("compose#".into(), 1, vec![f_val, g_val]));
                    }
                    BinOp::Seq => {
                        // a ; b — evaluate a for side effects, return b
                        self.eval(env, lhs)?;
                        return self.eval(env, rhs);
                    }
                    _ => {}
                }

                let l = self.eval(env, lhs)?;
                let r = self.eval(env, rhs)?;
                self.eval_binop(*op, l, r)
            }

            // ── Negation ────────────────────────────
            ExprKind::Neg(inner) => {
                match self.eval(env, inner)? {
                    Value::Int(n) => Ok(Value::Int(-n)),
                    Value::Float(n) => Ok(Value::Float(-n)),
                    _ => Err(err("Negation requires a number")),
                }
            }

            // ── Conditional ─────────────────────────
            ExprKind::Cond(guard, then_e, else_e) => {
                match self.eval(env, guard)? {
                    Value::Bool(true) => self.eval(env, then_e),
                    Value::Bool(false) => self.eval(env, else_e),
                    _ => Err(err("Condition must be Bool")),
                }
            }

            // ── List literal ────────────────────────
            ExprKind::List(elems) => {
                let vals: Vec<Value> = elems.iter()
                    .map(|e| self.eval(env, e))
                    .collect::<EResult<_>>()?;
                Ok(Value::List(vals))
            }

            // ── Range ───────────────────────────────
            ExprKind::Range(from, to) => {
                let f = self.eval(env, from)?;
                let t = self.eval(env, to)?;
                match (f, t) {
                    (Value::Int(a), Value::Int(b)) => {
                        Ok(Value::List((a..=b).map(Value::Int).collect()))
                    }
                    _ => Err(err("Range requires Int operands")),
                }
            }

            // ── List comprehension ──────────────────
            ExprKind::ListComp(body_expr, generators) => {
                self.eval_list_comp(env, body_expr, generators, 0)
            }

            // ── Block ───────────────────────────────
            ExprKind::Block(bindings, result) => {
                let mut local = env.child();
                for b in bindings {
                    let val = self.eval(&local, &b.value)?;
                    local.insert(b.name.clone(), val);
                }
                self.eval(&local, result)
            }

            // ── Record literal ───────────────────────
            ExprKind::Record(fields) => {
                let mut pairs = Vec::new();
                for (name, expr) in fields {
                    let val = self.eval(env, expr)?;
                    pairs.push((name.clone(), val));
                }
                Ok(Value::Record(pairs))
            }

            // ── Record update ────────────────────────
            ExprKind::RecordUpdate { base, updates } => {
                let base_val = self.eval(env, base)?;
                let Value::Record(mut fields) = base_val else {
                    return Err(err("record update requires a record as base"));
                };
                for (name, expr) in updates {
                    let val = self.eval(env, expr)?;
                    if let Some(entry) = fields.iter_mut().find(|(n, _)| n == name) {
                        entry.1 = val;
                    } else {
                        return Err(err(format!("field '{}' not found in record", name)));
                    }
                }
                Ok(Value::Record(fields))
            }

            // ── Field access ────────────────────────
            ExprKind::Field(obj, field) => {
                let obj_val = self.eval(env, obj)?;
                match obj_val {
                    Value::Record(fields) => {
                        fields.into_iter()
                            .find(|(name, _)| name == field)
                            .map(|(_, val)| val)
                            .ok_or_else(|| err(format!("Field '{}' not found in record", field)))
                    }
                    _ => Err(err(format!("Field access '{}' requires a record", field))),
                }
            }

            // ── Parenthesized ───────────────────────
            ExprKind::Paren(inner) => self.eval(env, inner),

            // ── Scope — structured concurrency ──────
            // Push a new handle-vec, evaluate body, pop and join all spawned threads.
            ExprKind::Scope(body) => {
                SCOPE_STACK.with(|s| s.borrow_mut().push(Vec::new()));
                let result = self.eval(env, body);
                let handles = SCOPE_STACK.with(|s| s.borrow_mut().pop().unwrap_or_default());
                for h in handles { let _ = h.join(); }
                result
            }

            // ── Spawn — launch in OS thread ──────────
            // Clone env + expr, spawn thread, register handle in nearest scope.
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
                    // spawn outside any scope: thread runs detached (handle dropped)
                });
                Ok(Value::Unit)
            }

            // ── Property generator — evaluated by test runner, not here ──
            ExprKind::Prop(vars, body) => {
                // When evaluated directly (not via test runner), treat as lambda
                // that returns Bool for a single random set. But test runner handles this.
                // Fallback: evaluate body with unbound vars → will error if vars used.
                // This path shouldn't normally be hit.
                let mut local = env.child();
                for var in vars {
                    local.insert(var.clone(), Value::Unit);
                }
                self.eval(&local, body)
            }

            // ── When — body when cond: if cond false, vacuously true ────
            ExprKind::When(body, cond) => {
                let cond_val = self.eval(env, cond)?;
                match cond_val {
                    Value::Bool(false) => Ok(Value::Bool(true)),
                    Value::Bool(true) => self.eval(env, body),
                    _ => Err(err("when: condition must be Bool")),
                }
            }
        }
    }

    // ── Apply a function value to an argument ────────

    pub fn apply(&mut self, func: Value, arg: Value) -> EResult<Value> {
        match func {
            Value::Closure { params, body, env } => {
                if params.len() == 1 {
                    let mut local = env.child();
                    self.bind_pattern(&params[0], &arg, &mut local)?;
                    self.eval(&local, &body)
                } else {
                    // Multi-param lambda: apply first param, return closure for rest
                    let mut local = env.child();
                    self.bind_pattern(&params[0], &arg, &mut local)?;
                    let rest = params[1..].to_vec();
                    Ok(Value::Closure {
                        params: rest,
                        body,
                        env: Arc::new(local),
                    })
                }
            }

            Value::Func { name, equations, env } => {
                // Case 1: Any equation has 0 patterns → evaluate body first
                // Handles: `f = \x -> x + a` where f is stored with 0 pats
                if equations.iter().any(|eq| eq.pats.is_empty()) {
                    // Find first 0-pattern equation and evaluate its body
                    for eq in &equations {
                        if eq.pats.is_empty() {
                            let mut local = env.child();
                            local.insert(name.clone(), Value::Func {
                                name: name.clone(),
                                equations: equations.clone(),
                                env: Arc::clone(&env),
                            });
                            let body_val = self.eval(&local, &eq.body)?;
                            return self.apply(body_val, arg);
                        }
                    }
                }

                // Case 2: All equations have >1 pattern → curry (bind first arg)
                // Handles: `map f [] = ... / map f (x:xs) = ...`
                let all_multi = equations.iter().all(|eq| eq.pats.len() > 1);

                if all_multi {
                    // Bind first pattern from each equation that matches,
                    // build new Func with remaining patterns for all equations
                    let mut local = env.child();
                    // Only insert self-reference if not already in env.
                    // The original full-arity function is in the parent env (set by eval_program).
                    // Inserting the reduced-arity version would shadow it and break recursion.
                    if env.lookup(&name).is_none() {
                        local.insert(name.clone(), Value::Func {
                            name: name.clone(),
                            equations: equations.clone(),
                            env: Arc::clone(&env),
                        });
                    }

                    // Filter: keep only equations whose first pattern matches.
                    // Store each equation's first-pattern bindings under unique
                    // hidden names to avoid variable collisions across equations.
                    let mut remaining: Vec<Equation> = Vec::new();
                    for eq in &equations {
                        let mut probe = local.child();
                        if self.try_bind_pattern(&eq.pats[0], &arg, &mut probe) {
                            let bindings: Vec<(String, Value)> = probe.bindings()
                                .into_iter()
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect();
                            let body = if bindings.is_empty() {
                                eq.body.clone()
                            } else {
                                // Store values under unique hidden names in shared env,
                                // then wrap body with let-bindings that alias them back.
                                let uid = next_curry_id();
                                let mut ast_bindings: Vec<Binding> = Vec::new();
                                for (i, (k, v)) in bindings.iter().enumerate() {
                                    let hidden = format!("__c{}_{}", uid, i);
                                    local.insert(hidden.clone(), v.clone());
                                    ast_bindings.push(Binding {
                                        name: k.clone(),
                                        value: Expr { kind: ExprKind::Var(hidden), span: eq.span },
                                        span: eq.span,
                                    });
                                }
                                Expr { kind: ExprKind::Block(ast_bindings, Box::new(eq.body.clone())), span: eq.span }
                            };
                            remaining.push(Equation {
                                pats: eq.pats[1..].to_vec(),
                                body,
                                span: eq.span,
                            });
                        }
                    }

                    if remaining.is_empty() {
                        return Err(err_no_match(format!(
                            "No matching pattern in function '{}' for argument {}",
                            name, arg
                        )));
                    }

                    return Ok(Value::Func {
                        name: name.clone(),
                        equations: remaining,
                        env: Arc::new(local),
                    });
                }

                // Case 3: Single-pattern equations → match and dispatch
                for eq in &equations {
                    if eq.pats.is_empty() {
                        continue;
                    }
                    let mut local = env.child();
                    // Only add self-reference if not already in env
                    // (after currying, env already has the original multi-arg version)
                    if env.lookup(&name).is_none() {
                        local.insert(name.clone(), Value::Func {
                            name: name.clone(),
                            equations: equations.clone(),
                            env: Arc::clone(&env),
                        });
                    }

                    if self.try_bind_pattern(&eq.pats[0], &arg, &mut local) {
                        if eq.pats.len() == 1 {
                            return self.eval(&local, &eq.body);
                        } else {
                            let remaining: Vec<Equation> = vec![Equation {
                                pats: eq.pats[1..].to_vec(),
                                body: eq.body.clone(),
                                span: eq.span,
                            }];
                            return Ok(Value::Func {
                                name: name.clone(),
                                equations: remaining,
                                env: Arc::new(local),
                            });
                        }
                    }
                }
                Err(err_no_match(format!("No matching pattern in function '{}' for argument {}", name, arg)))
            }

            Value::Builtin(name, arity) => {
                if arity == 1 {
                    self.call_builtin(&name, &[arg])
                } else {
                    Ok(Value::PartialBuiltin(name, arity - 1, vec![arg]))
                }
            }

            Value::PartialBuiltin(name, remaining, mut args) => {
                args.push(arg);
                if remaining == 1 {
                    self.call_builtin(&name, &args)
                } else {
                    Ok(Value::PartialBuiltin(name, remaining - 1, args))
                }
            }

            other => Err(err(format!("Cannot apply non-function: {}", other))),
        }
    }

    // ── Pattern matching ────────────────────────────

    fn bind_pattern(&self, pat: &Pat, val: &Value, env: &mut Env) -> EResult<()> {
        if !self.try_bind_pattern(pat, val, env) {
            Err(err_no_match(format!("Pattern match failed: {:?} vs {}", pat, val)))
        } else {
            Ok(())
        }
    }

    fn try_bind_pattern(&self, pat: &Pat, val: &Value, env: &mut Env) -> bool {
        match (pat, val) {
            (Pat::Wildcard, _) => true,

            (Pat::Var(name), _) => {
                env.insert(name.clone(), val.clone());
                true
            }

            (Pat::Lit(lit), val) => {
                &self.eval_lit(lit) == val
            }

            (Pat::Con(pname, ppats), Value::Con(vname, vfields)) => {
                if pname != vname || ppats.len() != vfields.len() {
                    return false;
                }
                ppats.iter().zip(vfields.iter())
                    .all(|(p, v)| self.try_bind_pattern(p, v, env))
            }

            // Empty list pattern: Con("[]", []) matches List([])
            (Pat::Con(name, pats), Value::List(elems)) if name == "[]" || name == "Nil" => {
                pats.is_empty() && elems.is_empty()
            }

            // Cons pattern: (x:xs) matches non-empty list
            (Pat::Cons(head, tail), Value::List(elems)) => {
                if elems.is_empty() {
                    return false;
                }
                let h = &elems[0];
                let t = Value::List(elems[1..].to_vec());
                self.try_bind_pattern(head, h, env)
                    && self.try_bind_pattern(tail, &t, env)
            }

            (Pat::Paren(inner), val) => self.try_bind_pattern(inner, val, env),

            (Pat::Record(pat_fields), Value::Record(val_fields)) => {
                for (pname, ppat) in pat_fields {
                    if let Some((_, val)) = val_fields.iter().find(|(n, _)| n == pname) {
                        if !self.try_bind_pattern(ppat, val, env) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            }

            _ => false,
        }
    }

    // ── List comprehension ──────────────────────────

    fn eval_list_comp(
        &mut self,
        env: &Env,
        body: &Expr,
        generators: &[Generator],
        gen_idx: usize,
    ) -> EResult<Value> {
        if gen_idx >= generators.len() {
            // All generators consumed — evaluate body
            let val = self.eval(env, body)?;
            return Ok(Value::List(vec![val]));
        }

        match &generators[gen_idx] {
            Generator::Bind(name, source_expr) => {
                let source = self.eval(env, source_expr)?;
                let list = match source {
                    Value::List(elems) => elems,
                    _ => return Err(err("List comprehension source must be a list")),
                };
                let mut result = Vec::new();
                for item in list {
                    let mut local = env.child();
                    local.insert(name.clone(), item);
                    match self.eval_list_comp(&local, body, generators, gen_idx + 1)? {
                        Value::List(vals) => result.extend(vals),
                        _ => unreachable!(),
                    }
                }
                Ok(Value::List(result))
            }
            Generator::Guard(guard_expr) => {
                match self.eval(env, guard_expr)? {
                    Value::Bool(true) => self.eval_list_comp(env, body, generators, gen_idx + 1),
                    Value::Bool(false) => Ok(Value::List(vec![])),
                    _ => Err(err("Guard must be Bool")),
                }
            }
        }
    }

    // ── Binary operators ────────────────────────────

    fn eval_binop(&self, op: BinOp, l: Value, r: Value) -> EResult<Value> {
        match op {
            // Arithmetic
            BinOp::Add => num_op(l, r, |a, b| a + b, |a, b| a + b),
            BinOp::Sub => num_op(l, r, |a, b| a - b, |a, b| a - b),
            BinOp::Mul => num_op(l, r, |a, b| a * b, |a, b| a * b),
            BinOp::Div => {
                let is_zero = matches!(&r, Value::Int(0))
                           || matches!(&r, Value::Float(f) if *f == 0.0);
                if is_zero {
                    Err(err_div_zero("Division by zero"))
                } else {
                    num_op(l, r, |a, b| a / b, |a, b| a / b)
                }
            }
            BinOp::Mod => {
                match (&l, &r) {
                    (Value::Int(a), Value::Int(b)) if *b != 0 => Ok(Value::Int(a % b)),
                    (Value::Int(_), Value::Int(0)) => Err(err_div_zero("Modulo by zero")),
                    _ => Err(err("% requires Int operands")),
                }
            }
            BinOp::Pow => {
                match (&l, &r) {
                    (Value::Int(a), Value::Int(b)) if *b >= 0 => {
                        let exp = u32::try_from(*b)
                            .map_err(|_| err("Exponent too large"))?;
                        a.checked_pow(exp)
                            .map(Value::Int)
                            .ok_or_else(|| err("Integer overflow in **"))
                    }
                    (Value::Int(_), Value::Int(_)) => Err(err("** requires non-negative exponent for Int")),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(*b))),
                    (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64).powf(*b))),
                    (Value::Float(a), Value::Int(b)) => {
                        if let Ok(exp) = i32::try_from(*b) {
                            Ok(Value::Float(a.powi(exp)))
                        } else {
                            Ok(Value::Float(a.powf(*b as f64)))
                        }
                    }
                    _ => Err(err("** requires numeric operands")),
                }
            }

            // Comparison
            BinOp::Eq => Ok(Value::Bool(l == r)),
            BinOp::Neq => Ok(Value::Bool(l != r)),
            BinOp::Lt => cmp_op(l, r, |o| o == std::cmp::Ordering::Less),
            BinOp::Gt => cmp_op(l, r, |o| o == std::cmp::Ordering::Greater),
            BinOp::Lte => cmp_op(l, r, |o| o != std::cmp::Ordering::Greater),
            BinOp::Gte => cmp_op(l, r, |o| o != std::cmp::Ordering::Less),

            // List
            BinOp::Concat => {
                match (l, r) {
                    (Value::List(mut a), Value::List(b)) => { a.extend(b); Ok(Value::List(a)) }
                    (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
                    _ => Err(err("++ requires two lists or two strings")),
                }
            }
            BinOp::Cons => {
                match r {
                    Value::List(mut elems) => { elems.insert(0, l); Ok(Value::List(elems)) }
                    _ => Err(err(": requires a list on the right")),
                }
            }

            BinOp::Compose => Err(err(">> composition should be desugared by parser")),
            BinOp::Pipe => Err(err("|> pipe should be handled in eval")),
            BinOp::And | BinOp::Or => Err(err("&&/|| should be short-circuited in eval")),
            BinOp::Seq => Err(err("; sequence should be short-circuited in eval")),
        }
    }

    // ── Literals ────────────────────────────────────

    fn eval_lit(&self, lit: &Lit) -> Value {
        match lit {
            Lit::Int(n) => Value::Int(*n),
            Lit::Float(n) => Value::Float(*n),
            Lit::Bool(b) => Value::Bool(*b),
            Lit::Str(s) => Value::Str(s.clone()),
            Lit::Char(c) => Value::Char(*c),
            Lit::Unit => Value::Unit,
        }
    }

    // ── Builtins ────────────────────────────────────

    fn builtin_env(&self) -> Env {
        let mut env = Env::new();
        // 0-arity: readline executes immediately on lookup (reads from stdin)
        env.insert("readline".to_string(), Value::Builtin("readline".to_string(), 0));
        for (name, arity) in &[
            ("print", 1), ("show", 1), ("show_bool", 1), ("length", 1),
            ("head", 1), ("tail", 1), ("even", 1), ("odd", 1),
            ("not", 1), ("sum", 1), ("filter", 2), ("map", 2),
            ("foldl", 3), ("zip", 2), ("index", 2), ("take", 2), ("drop", 2),
            ("reverse", 1),
            // Float math builtins
            ("sqrt", 1), ("floor", 1), ("ceil", 1), ("round", 1), ("abs", 1),
            // String builtins
            ("str_slice", 3), ("str_find", 3), ("str_starts_with", 2),
            ("str_trim", 1), ("str_len", 1), ("json_escape", 1), ("json_parse", 1), ("file_read", 1),
            // I/O builtins (for stress_server.sno)
            ("tcp_listen", 1), ("tcp_accept", 1),
            ("fd_readline", 1), ("fd_write", 2), ("fd_close", 1), ("fd_popen", 1),
            ("fd_open", 1), ("fd_open_write", 1),
            // Concurrency builtins (Phase C)
            ("send", 2), ("recv", 1),
            // Environment variables
            ("env", 1), ("env_or", 2),
            // Error: runtime panic with message
            ("error", 1),
        ] {
            env.insert(name.to_string(), Value::Builtin(name.to_string(), *arity));
        }
        // chan: 0-arity — creates a fresh channel on each evaluation
        env.insert("chan".to_string(), Value::Builtin("chan".to_string(), 0));
        // Nil / empty list constructors
        env.insert("Nil".into(), Value::List(vec![]));
        env.insert("None".into(), Value::Con("None".into(), vec![]));
        // CLI args: injected as `args : [String]`
        let arg_vals: Vec<Value> = self.args.iter().map(|s| Value::Str(s.clone())).collect();
        env.insert("args".into(), Value::List(arg_vals));
        env
    }

    fn call_builtin(&mut self, name: &str, args: &[Value]) -> EResult<Value> {
        match name {
            "readline" => {
                use std::io::BufRead;
                let stdin = std::io::stdin();
                let mut line = String::new();
                stdin.lock().read_line(&mut line)
                    .map_err(|e| err_io(format!("readline error: {}", e)))?;
                // Strip trailing newline
                if line.ends_with('\n') { line.pop(); }
                if line.ends_with('\r') { line.pop(); }
                Ok(Value::Str(line))
            }
            "print" => {
                let s = format!("{}", args[0]);
                self.output.push(s);
                Ok(Value::Unit)
            }
            "show" => Ok(Value::Str(format!("{}", args[0]))),
            "show_bool" => {
                let b = match &args[0] {
                    Value::Bool(b) => *b,
                    Value::Int(0) => false,
                    _ => true,
                };
                Ok(Value::Str(if b { "true".into() } else { "false".into() }))
            }
            "length" => match &args[0] {
                Value::List(l) => Ok(Value::Int(l.len() as i64)),
                Value::Str(s) => Ok(Value::Int(s.len() as i64)),
                _ => Err(err("length requires a list or string")),
            },
            "head" => match &args[0] {
                Value::List(l) if !l.is_empty() => Ok(l[0].clone()),
                Value::List(_) => Err(err("head of empty list")),
                _ => Err(err("head requires a list")),
            },
            "tail" => match &args[0] {
                Value::List(l) if !l.is_empty() => Ok(Value::List(l[1..].to_vec())),
                Value::List(_) => Err(err("tail of empty list")),
                _ => Err(err("tail requires a list")),
            },
            "even" => match &args[0] {
                Value::Int(n) => Ok(Value::Bool(n % 2 == 0)),
                _ => Err(err("even requires Int")),
            },
            "odd" => match &args[0] {
                Value::Int(n) => Ok(Value::Bool(n % 2 != 0)),
                _ => Err(err("odd requires Int")),
            },
            "not" => match &args[0] {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => Err(err("not requires Bool")),
            },
            "sum" => match &args[0] {
                Value::List(l) => {
                    let mut total = 0i64;
                    for v in l {
                        match v {
                            Value::Int(n) => total += n,
                            _ => return Err(err("sum requires a list of Int")),
                        }
                    }
                    Ok(Value::Int(total))
                }
                _ => Err(err("sum requires a list")),
            },
            "filter" => {
                let func = &args[0];
                match &args[1] {
                    Value::List(elems) => {
                        let mut result = Vec::new();
                        for e in elems {
                            let r = self.apply(func.clone(), e.clone())?;
                            if r == Value::Bool(true) {
                                result.push(e.clone());
                            }
                        }
                        Ok(Value::List(result))
                    }
                    _ => Err(err("filter requires a list")),
                }
            },
            "map" => {
                let func = &args[0];
                match &args[1] {
                    Value::List(elems) => {
                        let mut result = Vec::new();
                        for e in elems {
                            result.push(self.apply(func.clone(), e.clone())?);
                        }
                        Ok(Value::List(result))
                    }
                    _ => Err(err("map requires a list")),
                }
            },
            "foldl" => {
                let func = &args[0];
                let acc = &args[1];
                match &args[2] {
                    Value::List(elems) => {
                        let mut current = acc.clone();
                        for e in elems {
                            let partial = self.apply(func.clone(), current)?;
                            current = self.apply(partial, e.clone())?;
                        }
                        Ok(current)
                    }
                    _ => Err(err("foldl requires a list")),
                }
            },
            "zip" => {
                match (&args[0], &args[1]) {
                    (Value::List(a), Value::List(b)) => {
                        let pairs: Vec<Value> = a.iter().zip(b.iter())
                            .map(|(x, y)| Value::List(vec![x.clone(), y.clone()]))
                            .collect();
                        Ok(Value::List(pairs))
                    }
                    _ => Err(err("zip requires two lists")),
                }
            },
            "index" => {
                let idx = match &args[0] {
                    Value::Int(n) => *n,
                    _ => return Err(err("index requires Int as first argument")),
                };
                match &args[1] {
                    Value::List(l) => {
                        if idx < 0 || idx as usize >= l.len() {
                            Err(err(format!("index {} out of bounds (length {})", idx, l.len())))
                        } else {
                            Ok(l[idx as usize].clone())
                        }
                    }
                    _ => Err(err("index requires a list")),
                }
            },
            "take" => {
                let n = match &args[0] {
                    Value::Int(n) => (*n).max(0) as usize,
                    _ => return Err(err("take requires Int as first argument")),
                };
                match &args[1] {
                    Value::List(l) => Ok(Value::List(l.iter().take(n).cloned().collect())),
                    _ => Err(err("take requires a list")),
                }
            },
            "drop" => {
                let n = match &args[0] {
                    Value::Int(n) => (*n).max(0) as usize,
                    _ => return Err(err("drop requires Int as first argument")),
                };
                match &args[1] {
                    Value::List(l) => Ok(Value::List(l.iter().skip(n).cloned().collect())),
                    _ => Err(err("drop requires a list")),
                }
            },
            "reverse" => match &args[0] {
                Value::List(l) => {
                    let mut rev = l.clone();
                    rev.reverse();
                    Ok(Value::List(rev))
                }
                _ => Err(err("reverse requires a list")),
            },
            "compose#" => {
                // args[0]=f, args[1]=g, args[2]=x  →  g (f x)
                let fx = self.apply(args[0].clone(), args[2].clone())?;
                self.apply(args[1].clone(), fx)
            }
            "sqrt" => match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.sqrt())),
                Value::Int(n) => Ok(Value::Float((*n as f64).sqrt())),
                _ => Err(err("sqrt requires Float")),
            },
            "floor" => match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.floor())),
                _ => Err(err("floor requires Float")),
            },
            "ceil" => match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.ceil())),
                _ => Err(err("ceil requires Float")),
            },
            "round" => match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.round())),
                _ => Err(err("round requires Float")),
            },
            "abs" => match &args[0] {
                Value::Int(n) => Ok(Value::Int(n.abs())),
                Value::Float(f) => Ok(Value::Float(f.abs())),
                _ => Err(err("abs requires Int or Float")),
            },
            // ── String builtins ─────────────────────────────────
            "str_slice" => {
                // Byte-based (correct for ASCII HTTP headers)
                let s = sval(&args[0])?.as_bytes().to_vec();
                let from = (ival(&args[1])?.max(0) as usize).min(s.len());
                let to   = (ival(&args[2])?.max(0) as usize).min(s.len()).max(from);
                Ok(Value::Str(String::from_utf8_lossy(&s[from..to]).into_owned()))
            }
            "str_find" => {
                // Byte-based; returns -1 if not found
                let s   = sval(&args[0])?.as_bytes().to_vec();
                let sub = sval(&args[1])?.as_bytes().to_vec();
                let from = (ival(&args[2])?.max(0) as usize).min(s.len());
                if sub.is_empty() { return Ok(Value::Int(from as i64)); }
                for i in from..=s.len().saturating_sub(sub.len()) {
                    if s[i..].starts_with(&sub) { return Ok(Value::Int(i as i64)); }
                }
                Ok(Value::Int(-1))
            }
            "str_starts_with" => {
                let s = sval(&args[0])?;
                let p = sval(&args[1])?;
                Ok(Value::Bool(s.starts_with(p)))
            }
            "str_trim" => {
                Ok(Value::Str(sval(&args[0])?.trim().to_string()))
            }
            "str_len" => {
                Ok(Value::Int(sval(&args[0])?.len() as i64))
            }
            "json_escape" => {
                let s = sval(&args[0])?
                    .replace('\\', "\\\\")
                    .replace('"',  "\\\"")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t");
                Ok(Value::Str(s))
            }
            "json_parse" => {
                let s = sval(&args[0])?;
                match json_parse_value(s.as_bytes(), 0) {
                    Ok((val, pos)) => {
                        // Skip trailing whitespace
                        let end = skip_ws(s.as_bytes(), pos);
                        if end != s.len() {
                            Ok(Value::Con("Err".into(), vec![
                                Value::Str(format!("trailing content at position {}", end)),
                            ]))
                        } else {
                            Ok(Value::Con("Ok".into(), vec![val]))
                        }
                    }
                    Err(msg) => Ok(Value::Con("Err".into(), vec![Value::Str(msg)])),
                }
            }
            "file_read" => {
                let path = sval(&args[0])?;
                std::fs::read_to_string(path)
                    .map(Value::Str)
                    .map_err(|e| err_io(format!("file_read: {}", e)))
            }

            // ── Environment variables ────────────────────────────
            "env" => {
                let name = sval(&args[0])?;
                Ok(Value::Str(std::env::var(name).unwrap_or_default()))
            }
            "env_or" => {
                let name = sval(&args[0])?;
                let default = sval(&args[1])?;
                Ok(Value::Str(std::env::var(name).unwrap_or_else(|_| default.to_string())))
            }

            // ── I/O: TCP + process ───────────────────────────────
            "tcp_listen" => {
                let port = ival(&args[0])?;
                let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
                    .map_err(|e| err_io(format!("tcp_listen({}): {}", port, e)))?;
                let fd = next_io_fd();
                IO_LISTENERS.with(|l| l.borrow_mut().insert(fd, listener));
                Ok(Value::Int(fd))
            }
            "tcp_accept" => {
                let server_fd = ival(&args[0])?;
                let stream = IO_LISTENERS.with(|l| {
                    l.borrow().get(&server_fd)
                        .and_then(|lst| lst.accept().ok().map(|(s, _)| s))
                }).ok_or_else(|| err_io(format!("tcp_accept: bad fd {}", server_fd)))?;
                let writer = stream.try_clone()
                    .map_err(|e| err_io(format!("tcp_accept clone: {}", e)))?;
                let client_fd = next_io_fd();
                IO_READERS.with(|r| r.borrow_mut().insert(client_fd,
                    Box::new(BufReader::new(stream)) as Box<dyn BufRead>));
                IO_WRITERS.with(|w| w.borrow_mut().insert(client_fd,
                    Box::new(writer) as Box<dyn Write>));
                Ok(Value::Int(client_fd))
            }
            "fd_readline" => {
                let fd = ival(&args[0])?;
                let line = IO_READERS.with(|r| {
                    let mut map = r.borrow_mut();
                    map.get_mut(&fd).map(|reader| {
                        let mut line = String::new();
                        match reader.read_line(&mut line) {
                            Ok(0) | Err(_) => String::new(),
                            Ok(_) => {
                                if line.ends_with('\n') { line.pop(); }
                                if line.ends_with('\r') { line.pop(); }
                                line
                            }
                        }
                    }).unwrap_or_default()
                });
                Ok(Value::Str(line))
            }
            "fd_write" => {
                let fd = ival(&args[0])?;
                let s  = sval(&args[1])?;
                IO_WRITERS.with(|w| {
                    let mut map = w.borrow_mut();
                    if let Some(writer) = map.get_mut(&fd) {
                        let _ = writer.write_all(s.as_bytes());
                        let _ = writer.flush();
                    }
                });
                Ok(Value::Unit)
            }
            "fd_close" => {
                let fd = ival(&args[0])?;
                IO_READERS.with(|r| r.borrow_mut().remove(&fd));
                IO_WRITERS.with(|w| w.borrow_mut().remove(&fd));
                IO_LISTENERS.with(|l| l.borrow_mut().remove(&fd));
                IO_CHILDREN.with(|c| {
                    if let Some(mut child) = c.borrow_mut().remove(&fd) {
                        let _ = child.wait();
                    }
                });
                Ok(Value::Unit)
            }
            "fd_popen" => {
                let cmd = sval(&args[0])?.to_string();
                let home = std::env::var("HOME").unwrap_or_default();
                let path = std::env::var("PATH").unwrap_or_default();
                let new_path = format!("{}/.cargo/bin:{}", home, path);
                let mut child = Command::new("sh")
                    .arg("-c").arg(&cmd)
                    .env("PATH", new_path)
                    .stdout(Stdio::piped())
                    .spawn()
                    .map_err(|e| err_io(format!("fd_popen: {}", e)))?;
                let stdout = child.stdout.take()
                    .ok_or_else(|| err_io("fd_popen: no stdout"))?;
                let fd = next_io_fd();
                IO_READERS.with(|r| r.borrow_mut().insert(fd,
                    Box::new(BufReader::new(stdout)) as Box<dyn BufRead>));
                IO_CHILDREN.with(|c| c.borrow_mut().insert(fd, child));
                Ok(Value::Int(fd))
            }

            // ── File I/O builtins (streaming) ────────────────────────────────────

            // fd_open: open file for reading (returns fd for fd_readline / fd_close)
            "fd_open" => {
                let path = sval(&args[0])?.to_string();
                let file = std::fs::File::open(&path)
                    .map_err(|e| err_io(format!("fd_open: {}: {}", path, e)))?;
                let fd = next_io_fd();
                IO_READERS.with(|r| r.borrow_mut().insert(fd,
                    Box::new(BufReader::new(file)) as Box<dyn BufRead>));
                Ok(Value::Int(fd))
            }

            // fd_open_write: open file for writing (returns fd for fd_write / fd_close)
            "fd_open_write" => {
                let path = sval(&args[0])?.to_string();
                let file = std::fs::File::create(&path)
                    .map_err(|e| err_io(format!("fd_open_write: {}: {}", path, e)))?;
                let fd = next_io_fd();
                IO_WRITERS.with(|w| w.borrow_mut().insert(fd,
                    Box::new(std::io::BufWriter::new(file)) as Box<dyn Write>));
                Ok(Value::Int(fd))
            }

            // ── Concurrency builtins (Phase C) ──────────────────────────────────

            // chan: create a fresh typed channel
            "chan" => {
                let (tx, rx) = mpsc::channel::<Value>();
                Ok(Value::Chan(Arc::new(ChanInner {
                    sender:   Mutex::new(tx),
                    receiver: Mutex::new(rx),
                })))
            }

            // send ch val → Unit
            "send" => match &args[0] {
                Value::Chan(c) => {
                    c.sender.lock()
                        .map_err(|_| err("send: channel poisoned"))?
                        .send(args[1].clone())
                        .map_err(|_| err("send: channel closed"))?;
                    Ok(Value::Unit)
                }
                _ => Err(err("send: first argument must be Chan")),
            },

            // recv ch → value
            "recv" => match &args[0] {
                Value::Chan(c) => c.receiver.lock()
                    .map_err(|_| err("recv: channel poisoned"))?
                    .recv()
                    .map_err(|_| err("recv: channel closed or disconnected")),
                _ => Err(err("recv: argument must be Chan")),
            },

            "error" => {
                let msg = match &args[0] {
                    Value::Str(s) => s.clone(),
                    v => format!("{}", v),
                };
                Err(err(msg))
            },
            name if name.starts_with("ctor:") => {
                let ctor_name = &name[5..];
                Ok(Value::Con(ctor_name.to_string(), args.to_vec()))
            },
            _ => Err(err(format!("Unknown builtin: {}", name))),
        }
    }
}

// ── Helpers ──────────────────────────────────────────

fn num_op(l: Value, r: Value, int_op: fn(i64, i64) -> i64, float_op: fn(f64, f64) -> f64) -> EResult<Value> {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(int_op(a, b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(float_op(a, b))),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(float_op(a as f64, b))),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(float_op(a, b as f64))),
        _ => Err(err("Arithmetic requires numbers")),
    }
}

fn sval(v: &Value) -> EResult<&str> {
    if let Value::Str(s) = v { Ok(s) }
    else { Err(err("expected String")) }
}

fn ival(v: &Value) -> EResult<i64> {
    if let Value::Int(n) = v { Ok(*n) }
    else { Err(err("expected Int")) }
}

fn cmp_op(l: Value, r: Value, pred: fn(std::cmp::Ordering) -> bool) -> EResult<Value> {
    l.partial_cmp(&r)
        .map(|o| Value::Bool(pred(o)))
        .ok_or_else(|| err("Cannot compare these values"))
}

// ── JSON Parser (recursive descent) ──────────────────

fn skip_ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && matches!(b[i], b' ' | b'\t' | b'\n' | b'\r') { i += 1; }
    i
}

fn json_parse_value(b: &[u8], pos: usize) -> Result<(Value, usize), String> {
    let i = skip_ws(b, pos);
    if i >= b.len() { return Err("unexpected end of input".into()); }
    match b[i] {
        b'n' => json_parse_null(b, i),
        b't' | b'f' => json_parse_bool(b, i),
        b'"' => json_parse_string(b, i),
        b'[' => json_parse_array(b, i),
        b'{' => json_parse_object(b, i),
        b'-' | b'0'..=b'9' => json_parse_number(b, i),
        c => Err(format!("unexpected character '{}' at position {}", c as char, i)),
    }
}

fn json_parse_null(b: &[u8], i: usize) -> Result<(Value, usize), String> {
    if b[i..].starts_with(b"null") {
        Ok((Value::Con("JNull".into(), vec![]), i + 4))
    } else {
        Err(format!("expected 'null' at position {}", i))
    }
}

fn json_parse_bool(b: &[u8], i: usize) -> Result<(Value, usize), String> {
    if b[i..].starts_with(b"true") {
        Ok((Value::Con("JBool".into(), vec![Value::Bool(true)]), i + 4))
    } else if b[i..].starts_with(b"false") {
        Ok((Value::Con("JBool".into(), vec![Value::Bool(false)]), i + 5))
    } else {
        Err(format!("expected 'true' or 'false' at position {}", i))
    }
}

fn json_parse_number(b: &[u8], i: usize) -> Result<(Value, usize), String> {
    let mut j = i;
    if j < b.len() && b[j] == b'-' { j += 1; }
    if j >= b.len() || !b[j].is_ascii_digit() {
        return Err(format!("expected digit at position {}", j));
    }
    while j < b.len() && b[j].is_ascii_digit() { j += 1; }
    let mut is_float = false;
    if j < b.len() && b[j] == b'.' {
        is_float = true;
        j += 1;
        while j < b.len() && b[j].is_ascii_digit() { j += 1; }
    }
    if j < b.len() && (b[j] == b'e' || b[j] == b'E') {
        is_float = true;
        j += 1;
        if j < b.len() && (b[j] == b'+' || b[j] == b'-') { j += 1; }
        while j < b.len() && b[j].is_ascii_digit() { j += 1; }
    }
    let s = std::str::from_utf8(&b[i..j]).unwrap();
    if is_float {
        let n: f64 = s.parse().map_err(|_| format!("invalid number at position {}", i))?;
        Ok((Value::Con("JNum".into(), vec![Value::Float(n)]), j))
    } else {
        let n: i64 = s.parse().map_err(|_| format!("invalid number at position {}", i))?;
        Ok((Value::Con("JNum".into(), vec![Value::Int(n)]), j))
    }
}

fn json_parse_string(b: &[u8], i: usize) -> Result<(Value, usize), String> {
    let (s, end) = json_parse_raw_string(b, i)?;
    Ok((Value::Con("JStr".into(), vec![Value::Str(s)]), end))
}

fn json_parse_raw_string(b: &[u8], i: usize) -> Result<(String, usize), String> {
    if b[i] != b'"' { return Err(format!("expected '\"' at position {}", i)); }
    let mut j = i + 1;
    let mut s = String::new();
    while j < b.len() {
        match b[j] {
            b'"' => return Ok((s, j + 1)),
            b'\\' => {
                j += 1;
                if j >= b.len() { return Err("unexpected end in string escape".into()); }
                match b[j] {
                    b'"'  => s.push('"'),
                    b'\\' => s.push('\\'),
                    b'/'  => s.push('/'),
                    b'n'  => s.push('\n'),
                    b'r'  => s.push('\r'),
                    b't'  => s.push('\t'),
                    b'b'  => s.push('\u{0008}'),
                    b'f'  => s.push('\u{000C}'),
                    c => { s.push('\\'); s.push(c as char); }
                }
                j += 1;
            }
            c => { s.push(c as char); j += 1; }
        }
    }
    Err("unterminated string".into())
}

fn json_parse_array(b: &[u8], i: usize) -> Result<(Value, usize), String> {
    let mut j = i + 1; // skip '['
    let mut elems = Vec::new();
    j = skip_ws(b, j);
    if j < b.len() && b[j] == b']' {
        return Ok((Value::Con("JArr".into(), vec![Value::List(vec![])]), j + 1));
    }
    loop {
        let (val, next) = json_parse_value(b, j)?;
        elems.push(val);
        j = skip_ws(b, next);
        if j >= b.len() { return Err("unterminated array".into()); }
        if b[j] == b']' { break; }
        if b[j] != b',' { return Err(format!("expected ',' or ']' at position {}", j)); }
        j += 1; // skip ','
    }
    Ok((Value::Con("JArr".into(), vec![Value::List(elems)]), j + 1))
}

fn json_parse_object(b: &[u8], i: usize) -> Result<(Value, usize), String> {
    let mut j = i + 1; // skip '{'
    let mut pairs = Vec::new();
    j = skip_ws(b, j);
    if j < b.len() && b[j] == b'}' {
        return Ok((Value::Con("JObj".into(), vec![Value::List(vec![])]), j + 1));
    }
    loop {
        j = skip_ws(b, j);
        if j >= b.len() || b[j] != b'"' {
            return Err(format!("expected string key at position {}", j));
        }
        let (key, next) = json_parse_raw_string(b, j)?;
        j = skip_ws(b, next);
        if j >= b.len() || b[j] != b':' {
            return Err(format!("expected ':' at position {}", j));
        }
        j += 1; // skip ':'
        let (val, next) = json_parse_value(b, j)?;
        pairs.push(Value::Con("MkPair".into(), vec![Value::Str(key), val]));
        j = skip_ws(b, next);
        if j >= b.len() { return Err("unterminated object".into()); }
        if b[j] == b'}' { break; }
        if b[j] != b',' { return Err(format!("expected ',' or '}}' at position {}", j)); }
        j += 1; // skip ','
    }
    pairs.sort_by(|a, b| {
        let ka = if let Value::Con(_, ref args) = a { if let Value::Str(ref s) = args[0] { s.as_str() } else { "" } } else { "" };
        let kb = if let Value::Con(_, ref args) = b { if let Value::Str(ref s) = args[0] { s.as_str() } else { "" } } else { "" };
        ka.cmp(kb)
    });
    Ok((Value::Con("JObj".into(), vec![Value::List(pairs)]), j + 1))
}
