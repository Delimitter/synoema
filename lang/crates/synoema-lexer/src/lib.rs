// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

//! # synoema-lexer
//! Lexical analyzer for the Synoema programming language.
//!
//! All operators are designed to be single BPE tokens in cl100k_base.

pub mod token;
pub mod scanner;
pub mod layout;

pub use token::{Token, SpannedToken, Span, Pos};
pub use scanner::{Scanner, LexError};

/// Tokenize Synoema source code, including layout (INDENT/DEDENT).
pub fn lex(source: &str) -> Result<Vec<SpannedToken>, LexError> {
    let mut scanner = Scanner::new(source);
    let raw = scanner.scan_all()?;
    Ok(layout::apply_layout(raw))
}

/// Tokenize without layout processing (for debugging).
pub fn lex_raw(source: &str) -> Result<Vec<SpannedToken>, LexError> {
    let mut scanner = Scanner::new(source);
    scanner.scan_all()
}

/// Extract just the token types (without spans) for easy testing.
pub fn lex_tokens(source: &str) -> Result<Vec<Token>, LexError> {
    Ok(lex(source)?.into_iter().map(|st| st.token).collect())
}

#[cfg(test)]
mod tests;
