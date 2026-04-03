//! Type error definitions for Synoema.

use crate::types::{Type, TyVarId};
use synoema_lexer::Span;
use std::fmt;

/// A type error with an optional source span.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeError {
    pub kind: TypeErrorKind,
    pub span: Option<Span>,
}

impl TypeError {
    pub fn new(kind: TypeErrorKind, span: Option<Span>) -> Self {
        Self { kind, span }
    }

    /// Create a TypeError with no span (e.g. from unification).
    pub fn bare(kind: TypeErrorKind) -> Self {
        Self { kind, span: None }
    }

    /// Convenience: wrap a message string.
    pub fn other(msg: impl Into<String>) -> Self {
        Self::bare(TypeErrorKind::Other(msg.into()))
    }

    /// Attach a span to an existing error.
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    /// Attach a span only if not already set (preserves more precise inner span).
    pub fn or_span(mut self, span: Span) -> Self {
        if self.span.is_none() {
            self.span = Some(span);
        }
        self
    }
}

/// The kind of type error (what went wrong).
#[derive(Debug, Clone, PartialEq)]
pub enum TypeErrorKind {
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
    /// Linear variable used more than once
    LinearDuplicate { name: String },
    /// Linear variable never used (must be used exactly once)
    LinearUnused { name: String },
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            TypeErrorKind::Mismatch { expected, found } =>
                write!(f, "Type mismatch: expected {}, found {}", expected, found),
            TypeErrorKind::InfiniteType { var, ty } =>
                write!(f, "Infinite type: {} occurs in {}", Type::Var(*var), ty),
            TypeErrorKind::Unbound { name } =>
                write!(f, "Undefined variable: {}", name),
            TypeErrorKind::UnboundType { name } =>
                write!(f, "Undefined type: {}", name),
            TypeErrorKind::ArityMismatch { name, expected, found } =>
                write!(f, "{}: expected {} arguments, got {}", name, expected, found),
            TypeErrorKind::PatternMismatch { message } =>
                write!(f, "Pattern error: {}", message),
            TypeErrorKind::Other(msg) =>
                write!(f, "Type error: {}", msg),
            TypeErrorKind::LinearDuplicate { name } =>
                write!(f, "Linear variable '{}' used more than once", name),
            TypeErrorKind::LinearUnused { name } =>
                write!(f, "Linear variable '{}' must be used exactly once (never used)", name),
        }
    }
}

impl std::error::Error for TypeError {}
