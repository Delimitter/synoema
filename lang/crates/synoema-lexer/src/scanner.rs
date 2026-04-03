// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

use crate::token::*;

#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub message: String,
    pub pos: Pos,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lex error at {}:{}: {}", self.pos.line, self.pos.col, self.message)
    }
}

pub struct Scanner<'src> {
    src: &'src [u8],
    pos: usize,
    line: u32,
    col: u32,
    /// Stack of brace depths for nested string interpolations.
    /// Non-empty means we are inside `${...}` of an interpolated string.
    interp_stack: Vec<usize>,
    /// Buffered tokens (FIFO) produced by interpolation scanning.
    pending_tokens: Vec<SpannedToken>,
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str) -> Self {
        Self { src: source.as_bytes(), pos: 0, line: 1, col: 1, interp_stack: Vec::new(), pending_tokens: Vec::new() }
    }

    fn at_end(&self) -> bool { self.pos >= self.src.len() }
    fn peek(&self) -> u8 { if self.at_end() { 0 } else { self.src[self.pos] } }
    fn peek_at(&self, off: usize) -> u8 {
        let i = self.pos + off;
        if i >= self.src.len() { 0 } else { self.src[i] }
    }

    fn advance(&mut self) -> u8 {
        let ch = self.src[self.pos];
        self.pos += 1;
        if ch == b'\n' { self.line += 1; self.col = 1; } else { self.col += 1; }
        ch
    }

    fn current_pos(&self) -> Pos { Pos { line: self.line, col: self.col, offset: self.pos as u32 } }
    fn make_span(&self, start: Pos) -> Span { Span::new(start, self.current_pos()) }
    fn error(&self, msg: impl Into<String>) -> LexError { LexError { message: msg.into(), pos: self.current_pos() } }
    fn match_char(&mut self, expected: u8) -> bool {
        if !self.at_end() && self.peek() == expected { self.advance(); true } else { false }
    }

    fn skip_spaces(&mut self) {
        while !self.at_end() && (self.peek() == b' ' || self.peek() == b'\t') { self.advance(); }
    }

    fn skip_comment(&mut self) {
        while !self.at_end() && self.peek() != b'\n' { self.advance(); }
    }

    fn scan_doc_comment(&mut self) -> String {
        // Skip optional leading space after ---
        if !self.at_end() && self.peek() == b' ' { self.advance(); }
        let start = self.pos;
        while !self.at_end() && self.peek() != b'\n' { self.advance(); }
        std::str::from_utf8(&self.src[start..self.pos]).unwrap().trim_end().to_string()
    }

    fn scan_number(&mut self, start: Pos) -> Result<SpannedToken, LexError> {
        let ns = self.pos - 1;
        while !self.at_end() && self.peek().is_ascii_digit() { self.advance(); }
        if self.peek() == b'.' && self.peek_at(1).is_ascii_digit() {
            self.advance();
            while !self.at_end() && self.peek().is_ascii_digit() { self.advance(); }
            let s = std::str::from_utf8(&self.src[ns..self.pos]).unwrap();
            let v: f64 = s.parse().map_err(|_| self.error("invalid float"))?;
            Ok(SpannedToken { token: Token::Float(v), span: self.make_span(start) })
        } else {
            let s = std::str::from_utf8(&self.src[ns..self.pos]).unwrap();
            let v: i64 = s.parse().map_err(|_| self.error("invalid integer"))?;
            Ok(SpannedToken { token: Token::Int(v), span: self.make_span(start) })
        }
    }

    /// Scan a string escape sequence, pushing the result to `s`.
    fn scan_string_escape(&mut self, s: &mut String) -> Result<(), LexError> {
        if self.at_end() { return Err(self.error("unterminated escape")); }
        match self.advance() {
            b'n' => s.push('\n'), b'r' => s.push('\r'), b't' => s.push('\t'),
            b'\\' => s.push('\\'), b'"' => s.push('"'), b'0' => s.push('\0'),
            b'$' => s.push('$'),
            e => return Err(self.error(format!("unknown escape \\{}", e as char))),
        }
        Ok(())
    }

    fn scan_string(&mut self, start: Pos) -> Result<SpannedToken, LexError> {
        let mut s = String::new();
        loop {
            if self.at_end() || self.peek() == b'\n' { return Err(self.error("unterminated string")); }
            // Check for interpolation: ${
            if self.peek() == b'$' && self.peek_at(1) == b'{' {
                self.advance(); // $
                self.advance(); // {
                self.interp_stack.push(0);
                let interp_span = self.make_span(start);
                self.pending_tokens.push(SpannedToken { token: Token::InterpStart, span: interp_span });
                if s.is_empty() {
                    // No text before ${ — return InterpStart directly
                    return Ok(self.pending_tokens.remove(0));
                } else {
                    // Return StringFragment, InterpStart is queued
                    return Ok(SpannedToken { token: Token::StringFragment(s), span: self.make_span(start) });
                }
            }
            let ch = self.advance();
            if ch == b'"' { break; }
            if ch == b'\\' {
                self.scan_string_escape(&mut s)?;
            } else { s.push(ch as char); }
        }
        Ok(SpannedToken { token: Token::Str(s), span: self.make_span(start) })
    }

    /// Resume scanning an interpolated string after `}` closes an interpolation.
    /// Emits InterpEnd, then continues scanning for more text/interpolations until `"`.
    fn scan_interp_resume(&mut self, start: Pos) -> Result<SpannedToken, LexError> {
        let mut result = vec![SpannedToken { token: Token::InterpEnd, span: self.make_span(start) }];
        let mut s = String::new();
        let frag_start = self.current_pos();
        loop {
            if self.at_end() || self.peek() == b'\n' { return Err(self.error("unterminated string")); }
            // Check for another interpolation
            if self.peek() == b'$' && self.peek_at(1) == b'{' {
                self.advance(); // $
                self.advance(); // {
                self.interp_stack.push(0);
                if !s.is_empty() {
                    result.push(SpannedToken { token: Token::StringFragment(s), span: self.make_span(frag_start) });
                }
                result.push(SpannedToken { token: Token::InterpStart, span: self.make_span(self.current_pos()) });
                // Queue all but first, return first
                let first = result.remove(0);
                self.pending_tokens.extend(result);
                return Ok(first);
            }
            let ch = self.advance();
            if ch == b'"' {
                // End of interpolated string
                if !s.is_empty() {
                    result.push(SpannedToken { token: Token::StringFragment(s), span: self.make_span(frag_start) });
                }
                let first = result.remove(0);
                self.pending_tokens.extend(result);
                return Ok(first);
            }
            if ch == b'\\' {
                self.scan_string_escape(&mut s)?;
            } else { s.push(ch as char); }
        }
    }

    fn scan_char_lit(&mut self, start: Pos) -> Result<SpannedToken, LexError> {
        if self.at_end() { return Err(self.error("unterminated char")); }
        let ch = self.advance();
        let c = if ch == b'\\' {
            if self.at_end() { return Err(self.error("unterminated char escape")); }
            match self.advance() {
                b'n' => '\n', b't' => '\t', b'\\' => '\\', b'\'' => '\'', b'0' => '\0',
                e => return Err(self.error(format!("unknown escape \\{}", e as char))),
            }
        } else { ch as char };
        if !self.match_char(b'\'') { return Err(self.error("unterminated char literal")); }
        Ok(SpannedToken { token: Token::Char(c), span: self.make_span(start) })
    }

    fn scan_identifier(&mut self, start: Pos, first: u8) -> SpannedToken {
        let id_start = self.pos - 1;
        while !self.at_end() && (self.peek().is_ascii_alphanumeric() || self.peek() == b'_') { self.advance(); }
        let text = std::str::from_utf8(&self.src[id_start..self.pos]).unwrap();
        let token = match text {
            "mod" => Token::KwMod, "use" => Token::KwUse, "import" => Token::KwImport,
            "trait" => Token::KwTrait, "impl" => Token::KwImpl,
            "type" => Token::KwType,
            "true" => Token::KwTrue, "false" => Token::KwFalse,
            "lazy" => Token::KwLazy,
            "scope" => Token::KwScope, "spawn" => Token::KwSpawn,
            "derive" => Token::KwDerive,
            "test" => Token::KwTest, "prop" => Token::KwProp,
            "when" => Token::KwWhen,
            _ if first.is_ascii_uppercase() => Token::UpperId(text.to_string()),
            _ => Token::LowerId(text.to_string()),
        };
        SpannedToken { token, span: self.make_span(start) }
    }

    pub fn scan_token(&mut self) -> Result<SpannedToken, LexError> {
        // Drain pending tokens first (from interpolation scanning)
        if !self.pending_tokens.is_empty() {
            return Ok(self.pending_tokens.remove(0));
        }
        self.skip_spaces();
        if self.at_end() {
            return Ok(SpannedToken { token: Token::Eof, span: self.make_span(self.current_pos()) });
        }
        let start = self.current_pos();
        let ch = self.advance();
        let token = match ch {
            b'\n' => Token::Newline,
            b'"'  => return self.scan_string(start),
            b'\'' => return self.scan_char_lit(start),
            b'0'..=b'9' => return self.scan_number(start),
            b'a'..=b'z' | b'A'..=b'Z' => return Ok(self.scan_identifier(start, ch)),
            b'_' => {
                if !self.at_end() && (self.peek().is_ascii_alphanumeric() || self.peek() == b'_') {
                    return Ok(self.scan_identifier(start, ch));
                }
                Token::Underscore
            }
            b'-' => if self.match_char(b'>') { Token::Arrow }
                    else if self.peek() == b'-' {
                        self.advance(); // consume second '-'
                        if self.peek() == b'-' {
                            // --- doc comment
                            self.advance(); // consume third '-'
                            Token::DocComment(self.scan_doc_comment())
                        } else {
                            // -- regular comment (stripped)
                            self.skip_comment();
                            Token::Newline
                        }
                    }
                    else if self.peek() == b'o' && !self.peek_at(1).is_ascii_alphanumeric() && self.peek_at(1) != b'_' {
                        self.advance(); Token::LinearArrow
                    }
                    else { Token::Minus },
            b'<' => if self.match_char(b'-') { Token::BackArrow }
                    else if self.match_char(b'=') { Token::Lte } else { Token::Lt },
            b'>' => if self.match_char(b'=') { Token::Gte }
                    else if self.match_char(b'>') { Token::Compose } else { Token::Gt },
            b'|' => if self.match_char(b'>') { Token::Pipe }
                    else if self.match_char(b'|') { Token::Or } else { Token::Bar },
            b'+' => if self.match_char(b'+') { Token::Concat } else { Token::Plus },
            b'=' => if self.match_char(b'=') { Token::Eq } else { Token::Assign },
            b'!' => if self.match_char(b'=') { Token::Neq }
                    else { return Err(self.error("unexpected '!', did you mean '!='?")); },
            b'&' => if self.match_char(b'&') { Token::And }
                    else { return Err(self.error("unexpected '&', did you mean '&&'?")); },
            b'.' => if self.match_char(b'.') { Token::DotDot } else { Token::Dot },
            b'*' => if self.match_char(b'*') { Token::StarStar } else { Token::Star },
            b'/' => Token::Slash, b'%' => Token::Percent,
            b'?' => Token::Question, b':' => Token::Colon, b'@' => Token::At,
            b'\\' => Token::Backslash, b',' => Token::Comma, b';' => Token::Semicolon,
            b'(' => Token::LParen, b')' => Token::RParen,
            b'[' => Token::LBracket, b']' => Token::RBracket,
            b'{' => {
                // Track brace depth inside interpolation
                if let Some(depth) = self.interp_stack.last_mut() {
                    *depth += 1;
                }
                Token::LBrace
            }
            b'}' => {
                // Check if this closes an interpolation
                if let Some(depth) = self.interp_stack.last_mut() {
                    if *depth > 0 {
                        *depth -= 1;
                        Token::RBrace
                    } else {
                        self.interp_stack.pop();
                        return self.scan_interp_resume(start);
                    }
                } else {
                    Token::RBrace
                }
            }
            _ => return Err(self.error(format!("unexpected character '{}'", ch as char))),
        };
        Ok(SpannedToken { token, span: self.make_span(start) })
    }

    pub fn scan_all(&mut self) -> Result<Vec<SpannedToken>, LexError> {
        let mut tokens = Vec::new();
        loop {
            let st = self.scan_token()?;
            let is_eof = st.token == Token::Eof;
            tokens.push(st);
            if is_eof { break; }
        }
        Ok(tokens)
    }
}
