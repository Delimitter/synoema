//! Tree-walking evaluator for Synoema.
//!
//! Implements big-step operational semantics from the Language Reference §5.
//! Strict (eager) evaluation — arguments are evaluated before substitution.

use crate::value::{Value, Env};
use synoema_parser::*;

/// Evaluation error
#[derive(Debug, Clone)]
pub struct EvalError {
    pub message: String,
}

impl EvalError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { message: msg.into() }
    }
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Runtime error: {}", self.message)
    }
}

type EResult<T> = Result<T, EvalError>;

fn err(msg: impl Into<String>) -> EvalError { EvalError::new(msg) }

/// The Synoema evaluator
pub struct Evaluator {
    /// Output buffer (for testing — captures print output)
    pub output: Vec<String>,
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator { output: Vec::new() }
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

        // Second pass: register all functions (to enable mutual recursion)
        // Prepend impl equations where applicable
        for decl in &program.decls {
            if let Decl::Func { name, equations, .. } = decl {
                let prepend = impl_eqs.remove(name).unwrap_or_default();
                let mut all_eqs = prepend;
                all_eqs.extend(equations.iter().cloned());
                let func = Value::Func {
                    name: name.clone(),
                    equations: all_eqs,
                    env: env.clone(),
                };
                env.insert(name.clone(), func);
            }
        }

        // Register standalone impl methods not covered by any Func decl
        for (method_name, equations) in impl_eqs {
            let func = Value::Func {
                name: method_name.clone(),
                equations,
                env: env.clone(),
            };
            env.insert(method_name, func);
        }

        // Update function closures to capture the complete environment
        // (enables mutual recursion)
        let snapshot = env.clone();
        for decl in &program.decls {
            if let Decl::Func { name, .. } = decl {
                if let Some(Value::Func { equations, .. }) = snapshot.lookup(name) {
                    let equations = equations.clone();
                    let func = Value::Func {
                        name: name.clone(),
                        equations,
                        env: snapshot.clone(),
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
                            env: snapshot.clone(),
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
                    .ok_or_else(|| err(format!("Undefined variable: {}", name)))?;
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
                    .ok_or_else(|| err(format!("Undefined constructor: {}", name)))
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
                    env: env.clone(),
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
                        env: local,
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
                                env: env.clone(),
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
                            env: env.clone(),
                        });
                    }

                    // For curried functions, the first pattern position must be
                    // consistent (e.g., all Var, or matching constructors).
                    // Bind using the first equation's first pattern:
                    for eq in &equations {
                        if self.try_bind_pattern(&eq.pats[0], &arg, &mut local) {
                            break;
                        }
                    }

                    // Build remaining equations with first pattern stripped
                    let remaining: Vec<Equation> = equations.iter().map(|eq| {
                        Equation {
                            pats: eq.pats[1..].to_vec(),
                            body: eq.body.clone(),
                            span: eq.span,
                        }
                    }).collect();

                    return Ok(Value::Func {
                        name: name.clone(),
                        equations: remaining,
                        env: local,
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
                            env: env.clone(),
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
                                env: local,
                            });
                        }
                    }
                }
                Err(err(format!("No matching pattern in function '{}' for argument {}", name, arg)))
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
            Err(err(format!("Pattern match failed: {:?} vs {}", pat, val)))
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
                match (&l, &r) {
                    (Value::Int(_), Value::Int(0)) => Err(err("Division by zero")),
                    (Value::Float(_), Value::Float(b)) if *b == 0.0 => Err(err("Division by zero")),
                    _ => num_op(l, r, |a, b| a / b, |a, b| a / b),
                }
            }
            BinOp::Mod => {
                match (&l, &r) {
                    (Value::Int(a), Value::Int(b)) if *b != 0 => Ok(Value::Int(a % b)),
                    (Value::Int(_), Value::Int(0)) => Err(err("Modulo by zero")),
                    _ => Err(err("% requires Int operands")),
                }
            }
            BinOp::Pow => {
                match (&l, &r) {
                    (Value::Int(a), Value::Int(b)) if *b >= 0 => Ok(Value::Int(a.pow(*b as u32))),
                    (Value::Int(_), Value::Int(_)) => Err(err("** requires non-negative exponent for Int")),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(*b))),
                    (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64).powf(*b))),
                    (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.powi(*b as i32))),
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
            ("print", 1), ("show", 1), ("length", 1),
            ("head", 1), ("tail", 1), ("even", 1), ("odd", 1),
            ("not", 1), ("sum", 1), ("filter", 2), ("map", 2),
            ("foldl", 3),
            // Float math builtins
            ("sqrt", 1), ("floor", 1), ("ceil", 1), ("round", 1), ("abs", 1),
        ] {
            env.insert(name.to_string(), Value::Builtin(name.to_string(), *arity));
        }
        // Nil / empty list constructors
        env.insert("Nil".into(), Value::List(vec![]));
        env.insert("None".into(), Value::Con("None".into(), vec![]));
        env
    }

    fn call_builtin(&mut self, name: &str, args: &[Value]) -> EResult<Value> {
        match name {
            "readline" => {
                use std::io::BufRead;
                let stdin = std::io::stdin();
                let mut line = String::new();
                stdin.lock().read_line(&mut line)
                    .map_err(|e| err(format!("readline error: {}", e)))?;
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

fn cmp_op(l: Value, r: Value, pred: fn(std::cmp::Ordering) -> bool) -> EResult<Value> {
    l.partial_cmp(&r)
        .map(|o| Value::Bool(pred(o)))
        .ok_or_else(|| err("Cannot compare these values"))
}
