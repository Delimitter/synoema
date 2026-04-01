use synoema_lexer::{Token, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl ParseError {
    pub fn new(msg: impl Into<String>, span: Span) -> Self {
        Self { message: msg.into(), span }
    }

    pub fn expected(expected: &str, got: &Token, span: Span) -> Self {
        Self::new(format!("Expected {}, got {}", expected, token_desc(got)), span)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error at {}:{}: {}", self.span.start.line, self.span.start.col, self.message)
    }
}

impl std::error::Error for ParseError {}

fn token_desc(t: &Token) -> String {
    match t {
        Token::LowerId(s) => format!("'{}'", s),
        Token::UpperId(s) => format!("'{}'", s),
        Token::Int(n) => format!("{}", n),
        Token::Float(n) => format!("{}", n),
        Token::Str(s) => format!("\"{}\"", s),
        Token::Eof => "end of file".into(),
        Token::Newline => "newline".into(),
        Token::Indent => "indent".into(),
        Token::Dedent => "dedent".into(),
        other => format!("{:?}", other),
    }
}
