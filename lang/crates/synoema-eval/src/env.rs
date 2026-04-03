// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

//! Environment: maps names to runtime values with nested scopes.

use std::collections::HashMap;
use crate::value::{Value, FuncDef};

/// Evaluation environment with scope chain.
#[derive(Debug, Clone)]
pub struct Env {
    /// Variable bindings in current scope
    bindings: HashMap<String, Value>,
    /// Named function definitions (for multi-equation pattern matching)
    functions: HashMap<String, FuncDef>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    /// Look up a variable by name.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.bindings.get(name)
    }

    /// Bind a name to a value.
    pub fn set(&mut self, name: String, val: Value) {
        self.bindings.insert(name, val);
    }

    /// Register a multi-equation function.
    pub fn set_func(&mut self, func: FuncDef) {
        self.functions.insert(func.name.clone(), func);
    }

    /// Look up a function definition.
    pub fn get_func(&self, name: &str) -> Option<&FuncDef> {
        self.functions.get(name)
    }

    /// Create a child scope extending this one (for let-blocks, lambdas).
    pub fn extend(&self) -> Env {
        self.clone()
    }

    /// Extend with multiple bindings.
    pub fn extend_with(&self, bindings: Vec<(String, Value)>) -> Env {
        let mut new_env = self.clone();
        for (k, v) in bindings {
            new_env.bindings.insert(k, v);
        }
        new_env
    }

    /// Snapshot current bindings as a vec (for closures).
    pub fn snapshot(&self) -> Vec<(String, Value)> {
        self.bindings.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
}
