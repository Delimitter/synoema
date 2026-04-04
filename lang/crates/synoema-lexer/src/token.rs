// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

/// Token types for the Synoema programming language.
/// Every operator is chosen to be a single BPE token in cl100k_base.

/// Position in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub line: u32,
    pub col: u32,
    pub offset: u32,
}

/// Span of source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

impl Span {
    pub fn new(start: Pos, end: Pos) -> Self { Self { start, end } }
    pub fn dummy() -> Self {
        let p = Pos { line: 0, col: 0, offset: 0 };
        Self { start: p, end: p }
    }
}

/// A token with its span
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Int(i64),
    Float(f64),
    Str(String),
    Char(char),

    // Identifiers
    LowerId(String),
    UpperId(String),

    // Keywords
    KwMod, KwUse, KwImport, KwTrait, KwImpl, KwTrue, KwFalse, KwLazy, KwType,
    KwScope, KwSpawn, KwDerive,
    KwTest, KwProp, KwWhen,

    // Operators (all BPE-aligned: 1 token each)
    Arrow,       // ->
    LinearArrow, // -o  (linear function type)
    BackArrow,   // <-
    Pipe,       // |>
    Concat,     // ++
    Compose,    // >>
    Eq,         // ==
    Neq,        // !=
    Lt, Gt, Lte, Gte, // < > <= >=
    And,        // &&
    Or,         // ||
    Plus, Minus, Star, StarStar, Slash, Percent,
    Question,   // ?
    Colon,      // :
    Dot,        // .
    DotDot,     // ..
    DotDotDot,  // ...
    Assign,     // =
    At,         // @
    Bar,        // |
    Backslash,  // \ (lambda)
    Underscore, // _
    Comma,      // ,
    Semicolon,  // ; (sequence)

    // Delimiters
    LParen, RParen, LBracket, RBracket,
    LBrace, RBrace,  // { }

    // Layout
    Newline, Indent, Dedent,

    // String interpolation
    StringFragment(String),  // text segment of interpolated string
    InterpStart,             // ${ — begins interpolation expression
    InterpEnd,               // } that closes interpolation (not regular })

    // Documentation
    DocComment(String),  // --- text (preserved in AST, unlike -- which is stripped)

    // Special
    Eof,
}

impl Token {
    pub fn is_keyword(&self) -> bool {
        matches!(self, Token::KwMod | Token::KwUse | Token::KwImport | Token::KwTrait
            | Token::KwImpl | Token::KwTrue | Token::KwFalse | Token::KwLazy
            | Token::KwType | Token::KwScope | Token::KwSpawn | Token::KwDerive
            | Token::KwTest | Token::KwProp | Token::KwWhen)
    }

    pub fn is_literal(&self) -> bool {
        matches!(self, Token::Int(_) | Token::Float(_) | Token::Str(_)
            | Token::Char(_) | Token::KwTrue | Token::KwFalse)
    }

    pub fn describe(&self) -> &'static str {
        match self {
            Token::Int(_) => "integer", Token::Float(_) => "float",
            Token::Str(_) => "string", Token::Char(_) => "char",
            Token::LowerId(_) => "identifier", Token::UpperId(_) => "constructor",
            Token::KwMod => "'mod'", Token::KwUse => "'use'", Token::KwImport => "'import'",
            Token::KwTrait => "'trait'", Token::KwImpl => "'impl'",
            Token::KwType => "'type'",
            Token::KwTrue => "'true'", Token::KwFalse => "'false'",
            Token::KwLazy => "'lazy'",
            Token::KwScope => "'scope'", Token::KwSpawn => "'spawn'",
            Token::KwDerive => "'deriving'",
            Token::KwTest => "'test'", Token::KwProp => "'prop'",
            Token::KwWhen => "'when'",
            Token::Arrow => "'->'", Token::LinearArrow => "'-o'", Token::BackArrow => "'<-'",
            Token::Pipe => "'|>'", Token::Concat => "'++'",
            Token::Compose => "'>>'",
            Token::Eq => "'=='", Token::Neq => "'!='",
            Token::Lt => "'<'", Token::Gt => "'>'",
            Token::Lte => "'<='", Token::Gte => "'>='",
            Token::And => "'&&'", Token::Or => "'||'",
            Token::Plus => "'+'", Token::Minus => "'-'",
            Token::Star => "'*'", Token::StarStar => "'**'", Token::Slash => "'/'",
            Token::Percent => "'%'", Token::Question => "'?'",
            Token::Colon => "':'", Token::Dot => "'.'",
            Token::DotDot => "'..'", Token::DotDotDot => "'...'", Token::Assign => "'='",
            Token::At => "'@'", Token::Bar => "'|'",
            Token::Backslash => "'\\'", Token::Underscore => "'_'",
            Token::Comma => "','",
            Token::Semicolon => "';'",
            Token::LParen => "'('", Token::RParen => "')'",
            Token::LBracket => "'['", Token::RBracket => "']'",
            Token::LBrace => "'{'", Token::RBrace => "'}'",
            Token::StringFragment(_) => "string fragment",
            Token::InterpStart => "'${'",
            Token::InterpEnd => "'}'",
            Token::DocComment(_) => "doc comment",
            Token::Newline => "newline", Token::Indent => "indent",
            Token::Dedent => "dedent", Token::Eof => "end of file",
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Int(n) => write!(f, "{n}"),
            Token::Float(n) => write!(f, "{n}"),
            Token::Str(s) => write!(f, "\"{s}\""),
            Token::Char(c) => write!(f, "'{c}'"),
            Token::StringFragment(s) => write!(f, "\"{s}\""),
            Token::InterpStart => write!(f, "${{"),
            Token::InterpEnd => write!(f, "}}"),
            Token::LowerId(s) | Token::UpperId(s) | Token::DocComment(s) => write!(f, "{s}"),
            _ => write!(f, "{}", self.describe()),
        }
    }
}
