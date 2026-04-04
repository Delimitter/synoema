// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

//! Runtime values for the Synoema tree-walking interpreter.

use std::fmt;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver};
use synoema_parser::{Pat, Expr, Equation};

/// Shared state for a typed channel (Phase C).
/// Wrapped in Arc so Value::Chan is Clone.
pub struct ChanInner {
    pub sender:   Mutex<Sender<Value>>,
    pub receiver: Mutex<Receiver<Value>>,
}

impl std::fmt::Debug for ChanInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChanInner {{ .. }}")
    }
}

/// Runtime value
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Char(char),
    /// List of values
    List(Vec<Value>),
    /// Constructor with tag and fields: `Just 42`, `None`
    Con(String, Vec<Value>),
    /// Closure: captured env + params + body
    Closure {
        params: Vec<Pat>,
        body: Expr,
        env: Arc<Env>,
    },
    /// Multi-equation function (pattern matching across equations)
    Func {
        name: String,
        equations: Vec<Equation>,
        env: Arc<Env>,
    },
    /// Built-in function
    Builtin(String, usize), // name, arity
    /// Partially applied builtin
    PartialBuiltin(String, usize, Vec<Value>), // name, remaining arity, accumulated args
    /// Record value: {name = "Alice", age = 30}
    Record(Vec<(String, Value)>),
    /// Unit (void)
    Unit,
    /// Typed channel (Phase C): send/recv values across threads.
    Chan(Arc<ChanInner>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Con(na, fa), Value::Con(nb, fb)) => na == nb && fa == fb,
            (Value::Record(a), Value::Record(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            // Channels are compared by pointer identity (two different chans are never equal)
            (Value::Chan(a), Value::Chan(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            (Value::Str(a), Value::Str(b)) => a.partial_cmp(b),
            (Value::Char(a), Value::Char(b)) => a.partial_cmp(b),
            (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => {
                if *n == (*n as i64) as f64 {
                    write!(f, "{:.1}", n)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::Bool(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
            Value::Char(c) => write!(f, "{}", c),
            Value::List(elems) => {
                write!(f, "[")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "{}", e)?;
                }
                write!(f, "]")
            }
            Value::Con(name, fields) => {
                write!(f, "{}", name)?;
                for fld in fields {
                    write!(f, " ")?;
                    match fld {
                        Value::Con(_, fs) if !fs.is_empty() => write!(f, "({})", fld)?,
                        _ => write!(f, "{}", fld)?,
                    }
                }
                Ok(())
            }
            Value::Record(fields) => {
                write!(f, "{{")?;
                for (i, (name, val)) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{} = {}", name, val)?;
                }
                write!(f, "}}")
            }
            Value::Closure { .. } => write!(f, "<closure>"),
            Value::Func { name, .. } => write!(f, "<fn {}>", name),
            Value::Builtin(name, _) => write!(f, "<builtin {}>", name),
            Value::PartialBuiltin(name, _, _) => write!(f, "<partial {}>", name),
            Value::Unit => write!(f, "()"),
            Value::Chan(_) => write!(f, "<chan>"),
        }
    }
}

/// Environment: chain of scopes for variable lookup
#[derive(Debug, Clone)]
pub struct Env {
    frames: Vec<std::collections::HashMap<String, Value>>,
}

impl Env {
    pub fn new() -> Self {
        Env { frames: vec![std::collections::HashMap::new()] }
    }

    pub fn lookup(&self, name: &str) -> Option<&Value> {
        for frame in self.frames.iter().rev() {
            if let Some(v) = frame.get(name) {
                return Some(v);
            }
        }
        None
    }

    pub fn insert(&mut self, name: String, val: Value) {
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(name, val);
        }
    }

    pub fn push_scope(&mut self) {
        self.frames.push(std::collections::HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }

    /// Create a child environment (for closures)
    pub fn child(&self) -> Self {
        let mut e = self.clone();
        e.push_scope();
        e
    }

    /// Return bindings from the topmost frame (useful after try_bind_pattern into a child).
    pub fn bindings(&self) -> &std::collections::HashMap<String, Value> {
        self.frames.last().unwrap()
    }

    /// Iterate over all bindings (most recent scope first)
    pub fn iter_all(&self) -> Vec<(String, Value)> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        for frame in self.frames.iter().rev() {
            for (k, v) in frame {
                if seen.insert(k.clone()) {
                    result.push((k.clone(), v.clone()));
                }
            }
        }
        result
    }
}
