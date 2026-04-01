//! Type error definitions for Synoema.

use crate::types::{Type, TyVarId};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    /// Two types could not be unified
    Mismatch { expected: Type, found: Type },
    /// Occurs check failure: infinite type
    InfiniteType { var: TyVarId, ty: Type },
    /// Undefined variable
    Unbound { name: String },
    /// Undefined type constructor
    UnboundType { name: String },
    /// Wrong number of arguments
    ArityMismatch { name: String, expected: usize, found: usize },
    /// Pattern match type error
    PatternMismatch { message: String },
    /// General error with message
    Other(String),
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::Mismatch { expected, found } =>
                write!(f, "Type mismatch: expected {}, found {}", expected, found),
            TypeError::InfiniteType { var, ty } =>
                write!(f, "Infinite type: {} occurs in {}", Type::Var(*var), ty),
            TypeError::Unbound { name } =>
                write!(f, "Undefined variable: {}", name),
            TypeError::UnboundType { name } =>
                write!(f, "Undefined type: {}", name),
            TypeError::ArityMismatch { name, expected, found } =>
                write!(f, "{}: expected {} arguments, got {}", name, expected, found),
            TypeError::PatternMismatch { message } =>
                write!(f, "Pattern error: {}", message),
            TypeError::Other(msg) =>
                write!(f, "Type error: {}", msg),
        }
    }
}

impl std::error::Error for TypeError {}
