// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

//! # synoema-parser
//! Parser for the Synoema programming language.
//!
//! Converts a token stream from `synoema-lexer` into an Abstract Syntax Tree.
//! Uses Pratt parsing for expressions and recursive descent for declarations.

pub mod ast;
pub mod derive;
pub mod error;
pub mod imports;
pub mod parser;

pub use ast::*;
pub use derive::{expand_derives, DeriveError};
pub use error::ParseError;
pub use imports::{resolve_imports, ImportError, ImportErrorCode};
pub use parser::Parser;

/// Parse Synoema source code into a Program AST.
/// Automatically expands `deriving` clauses into synthetic ImplDecl entries.
pub fn parse(source: &str) -> Result<Program, ParseError> {
    let tokens = synoema_lexer::lex(source)
        .map_err(|e| ParseError::new(e.message, synoema_lexer::Span::dummy()))?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let derive_errors = expand_derives(&mut program);
    if let Some(err) = derive_errors.first() {
        return Err(ParseError::new(err.to_string(), err.span));
    }
    Ok(program)
}

/// Parse with error recovery: returns partial AST + all errors found.
/// Useful for LLM workflows where all errors should be reported in one pass.
pub fn parse_recovering(source: &str) -> Result<(Program, Vec<ParseError>), ParseError> {
    let tokens = synoema_lexer::lex(source)
        .map_err(|e| ParseError::new(e.message, synoema_lexer::Span::dummy()))?;
    let mut parser = Parser::new(tokens);
    let (mut program, mut errors) = parser.parse_program_recovering();
    let derive_errors = expand_derives(&mut program);
    for err in derive_errors {
        errors.push(ParseError::new(err.to_string(), err.span));
    }
    Ok((program, errors))
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
