//! # synoema-parser
//! Parser for the Synoema programming language.
//!
//! Converts a token stream from `synoema-lexer` into an Abstract Syntax Tree.
//! Uses Pratt parsing for expressions and recursive descent for declarations.

pub mod ast;
pub mod error;
pub mod parser;

pub use ast::*;
pub use error::ParseError;
pub use parser::Parser;

/// Parse Synoema source code into a Program AST.
pub fn parse(source: &str) -> Result<Program, ParseError> {
    let tokens = synoema_lexer::lex(source)
        .map_err(|e| ParseError::new(e.message, synoema_lexer::Span::dummy()))?;
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

/// Parse a single expression (for REPL).
pub fn parse_expr(source: &str) -> Result<Expr, ParseError> {
    let tokens = synoema_lexer::lex(source)
        .map_err(|e| ParseError::new(e.message, synoema_lexer::Span::dummy()))?;
    let mut parser = Parser::new(tokens);
    parser.parse_expr()
}

#[cfg(test)]
mod tests;
