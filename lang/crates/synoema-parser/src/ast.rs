// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

//! Abstract Syntax Tree for Synoema.
//!
//! Every node carries a `Span` for error reporting.
//! The AST is produced by the parser and consumed by the type checker.

use synoema_lexer::Span;

// ── Literals ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    Int(i64),
    Float(f64),
    Str(String),
    Char(char),
    Bool(bool),
    Unit,
}

// ── Binary operators ─────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod, Pow,   // arithmetic
    Eq, Neq, Lt, Gt, Lte, Gte,    // comparison
    And, Or,                        // logic
    Concat,                         // ++
    Cons,                           // : (list cons)
    Pipe,                           // |>
    Compose,                        // >>
    Seq,                            // ; (sequence: eval left for effect, return right)
}

impl BinOp {
    pub fn symbol(&self) -> &'static str {
        match self {
            BinOp::Add => "+",  BinOp::Sub => "-",  BinOp::Mul => "*",
            BinOp::Div => "/",  BinOp::Mod => "%",  BinOp::Pow => "**",
            BinOp::Eq => "==",  BinOp::Neq => "!=",
            BinOp::Lt => "<",   BinOp::Gt => ">",
            BinOp::Lte => "<=", BinOp::Gte => ">=",
            BinOp::And => "&&", BinOp::Or => "||",
            BinOp::Concat => "++", BinOp::Cons => ":",
            BinOp::Pipe => "|>", BinOp::Compose => ">>",
            BinOp::Seq => ";",
        }
    }
}

// ── Patterns ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Pat {
    /// `_`
    Wildcard,
    /// Variable binding: `x`, `n`
    Var(String),
    /// Literal: `0`, `"hello"`, `true`
    Lit(Lit),
    /// Constructor application: `Just x`, `Cons h t`
    Con(String, Vec<Pat>),
    /// Cons pattern: `(x:xs)`
    Cons(Box<Pat>, Box<Pat>),
    /// Parenthesized: `(pat)`
    Paren(Box<Pat>),
    /// Record pattern: `{x = v, y = _}`
    Record(Vec<(String, Pat)>),
}

// ── Expressions ──────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    /// Literal value
    Lit(Lit),

    /// Variable reference: `x`, `foo`
    Var(String),

    /// Constructor: `Just`, `None`, `Cons`
    Con(String),

    /// Function application: `f x`
    App(Box<Expr>, Box<Expr>),

    /// Lambda: `\x y -> body`
    Lam(Vec<Pat>, Box<Expr>),

    /// Binary operator: `x + y`
    BinOp(BinOp, Box<Expr>, Box<Expr>),

    /// Unary minus: `-x`
    Neg(Box<Expr>),

    /// Conditional: `? cond -> then_e : else_e`
    Cond(Box<Expr>, Box<Expr>, Box<Expr>),

    /// Field access: `r.name`
    Field(Box<Expr>, String),

    /// Record literal: `{x = 3, y = 4}`
    Record(Vec<(String, Expr)>),

    /// List literal: `[1 2 3]`
    List(Vec<Expr>),

    /// List comprehension: `[expr | generators]`
    ListComp(Box<Expr>, Vec<Generator>),

    /// Range: `[1..10]`
    Range(Box<Expr>, Box<Expr>),

    /// Block with local bindings:
    /// ```axiom
    /// result =
    ///   a = 10
    ///   b = 20
    ///   a + b
    /// ```
    Block(Vec<Binding>, Box<Expr>),

    /// Parenthesized expression
    Paren(Box<Expr>),

    /// Structured concurrency scope: `scope { body }`
    /// All threads spawned within body are joined before scope returns.
    Scope(Box<Expr>),

    /// Spawn expression: `spawn expr`
    /// Runs expr in a new OS thread within the nearest enclosing scope.
    /// Returns Unit.
    Spawn(Box<Expr>),

    /// Property generator: `prop x y -> body`
    Prop(Vec<String>, Box<Expr>),

    /// Conditional property: `body when cond` (if cond false, test discarded)
    When(Box<Expr>, Box<Expr>),
}

/// Local binding inside a block: `name = expr`
#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

/// Generator in a list comprehension
#[derive(Debug, Clone, PartialEq)]
pub enum Generator {
    /// `x <- expr`  — draw from list
    Bind(String, Expr),
    /// `expr`       — boolean guard
    Guard(Expr),
}

// ── Type expressions ─────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct TypeExpr {
    pub kind: TypeExprKind,
    pub span: Span,
}

impl TypeExpr {
    pub fn new(kind: TypeExprKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeExprKind {
    /// Type variable: `a`, `b`
    Var(String),
    /// Named type: `Int`, `Bool`, `Maybe`
    Con(String),
    /// Function type: `a -> b`
    Arrow(Box<TypeExpr>, Box<TypeExpr>),
    /// Linear function type: `a -o b` (argument used exactly once)
    LinearArrow(Box<TypeExpr>, Box<TypeExpr>),
    /// Type application: `Maybe Int`, `List a`
    App(Box<TypeExpr>, Box<TypeExpr>),
    /// Parenthesized
    Paren(Box<TypeExpr>),
}

// ── Top-level declarations ───────────────────────────────

/// A single equation of a function: `fac 0 = 1`
#[derive(Debug, Clone, PartialEq)]
pub struct Equation {
    pub pats: Vec<Pat>,
    pub body: Expr,
    pub span: Span,
}

/// Type signature: `add : Int -> Int -> Int`
#[derive(Debug, Clone, PartialEq)]
pub struct TypeSig {
    pub name: String,
    pub ty: TypeExpr,
    pub span: Span,
}

/// Algebraic data type variant: `Just a`, `None`, `Circle Float`
#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<TypeExpr>,
    pub span: Span,
}

/// Top-level declaration
#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    /// Function definition (one or more equations):
    /// ```axiom
    /// fac 0 = 1
    /// fac n = n * fac (n - 1)
    /// ```
    Func {
        name: String,
        equations: Vec<Equation>,
        span: Span,
        doc: Vec<String>,
    },

    /// Type signature: `add : Int -> Int -> Int`
    TypeSig(TypeSig),

    /// ADT definition: `Maybe a = Just a | None`
    TypeDef {
        name: String,
        params: Vec<String>,
        variants: Vec<Variant>,
        span: Span,
        doc: Vec<String>,
        derives: Vec<String>,
    },

    /// Type class declaration: `trait Show a\n  show : a -> String`
    TraitDecl {
        name: String,
        ty_param: String,
        methods: Vec<TypeSig>,
        span: Span,
        doc: Vec<String>,
    },

    /// Type class implementation: `impl Show Color\n  show Red = "Red"`
    ImplDecl {
        trait_name: String,
        ty_name: String,
        methods: Vec<(String, Vec<Equation>)>,
        span: Span,
    },

    /// Type alias: `type Pos = {x : Int, y : Int}`
    TypeAlias {
        name: String,
        params: Vec<String>,
        body: TypeExpr,
        span: Span,
    },

    /// Test declaration: `test "name" = expr`
    Test {
        name: String,
        body: Expr,
        span: Span,
    },
}

// ── Module declarations ──────────────────────────────────

/// A `mod Name` block containing declarations
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDecl {
    pub name: String,
    pub body: Vec<Decl>,
    pub doc: Vec<String>,
}

/// A `use Module (name1 name2)` import
#[derive(Debug, Clone, PartialEq)]
pub struct UseDecl {
    pub module: String,
    pub names: Vec<String>,
    pub span: Span,
}

/// An `import "path.sno"` file import
#[derive(Debug, Clone, PartialEq)]
pub struct ImportDecl {
    pub path: String,
    pub span: Span,
}

// ── Program ──────────────────────────────────────────────

/// A complete Synoema program
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub imports: Vec<ImportDecl>,
    pub decls: Vec<Decl>,
    pub modules: Vec<ModuleDecl>,
    pub uses: Vec<UseDecl>,
}
