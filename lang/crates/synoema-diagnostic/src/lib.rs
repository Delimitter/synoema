//! # synoema-diagnostic
//! Structured error reporting for the Synoema programming language.
//!
//! Provides a unified `Diagnostic` type with renderers for human-readable
//! and JSON output, optimized for LLM consumption in the generate → check → fix loop.

use synoema_lexer::Span;
use std::fmt;

// ── Severity ────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
        }
    }
}

// ── Label ───────────────────────────────────────────────

/// An annotated source span (secondary location context).
#[derive(Debug, Clone)]
pub struct Label {
    pub span: Span,
    pub message: String,
}

// ── Diagnostic ──────────────────────────────────────────

/// A structured compiler diagnostic.
///
/// Designed for two consumers:
/// - **Human** (terminal): `render_human(&self, source)` → annotated source snippet
/// - **LLM** (API/tool): `render_json(&self)` → machine-parseable JSON
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Stable string code: "type_mismatch", "unbound_var", etc.
    pub code: &'static str,
    /// Error or warning.
    pub severity: Severity,
    /// Main message (one line, no location info).
    pub message: String,
    /// Primary source span (where the error occurred).
    pub span: Option<Span>,
    /// Additional annotated spans.
    pub labels: Vec<Label>,
    /// Extra context lines (e.g. "expected: Int", "found: Bool").
    pub notes: Vec<String>,
}

impl Diagnostic {
    /// Create an error diagnostic.
    pub fn error(code: &'static str, message: impl Into<String>) -> Self {
        Diagnostic {
            code,
            severity: Severity::Error,
            message: message.into(),
            span: None,
            labels: vec![],
            notes: vec![],
        }
    }

    /// Create a warning diagnostic.
    pub fn warning(code: &'static str, message: impl Into<String>) -> Self {
        Diagnostic {
            code,
            severity: Severity::Warning,
            message: message.into(),
            span: None,
            labels: vec![],
            notes: vec![],
        }
    }

    /// Attach a primary span.
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    /// Attach a primary span if present.
    pub fn maybe_span(mut self, span: Option<Span>) -> Self {
        self.span = span;
        self
    }

    /// Add a secondary label.
    pub fn with_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label { span, message: message.into() });
        self
    }

    /// Add a context note.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
}

// ── Display (default = human-readable, no source) ───────

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]", self.severity, self.code)?;
        if let Some(span) = &self.span {
            write!(f, " at {}:{}", span.start.line, span.start.col)?;
        }
        write!(f, ": {}", self.message)
    }
}

impl std::error::Error for Diagnostic {}

// ── Human renderer (with source snippet) ────────────────

/// Render a diagnostic with source context.
///
/// ```text
/// error[type_mismatch] at 3:5: expected Int, found Bool
///   3 | main = 1 + true
///                  ^^^^
///   = note: expected: Int
///   = note: found: Bool
/// ```
pub fn render_human(diag: &Diagnostic, source: Option<&str>) -> String {
    let mut out = String::new();

    // Header
    out.push_str(&format!("{}[{}]", diag.severity, diag.code));
    if let Some(span) = &diag.span {
        out.push_str(&format!(" at {}:{}", span.start.line, span.start.col));
    }
    out.push_str(&format!(": {}\n", diag.message));

    // Source snippet
    if let (Some(span), Some(src)) = (&diag.span, source) {
        if span.start.line > 0 {
            if let Some(line_text) = get_source_line(src, span.start.line) {
                let line_num = format!("{}", span.start.line);
                let padding = " ".repeat(line_num.len());
                out.push_str(&format!("{} |\n", padding));
                out.push_str(&format!("{} | {}\n", line_num, line_text));

                // Underline
                let col = (span.start.col as usize).saturating_sub(1);
                let end_col = if span.start.line == span.end.line && span.end.col > span.start.col {
                    span.end.col as usize
                } else {
                    // Underline to end of interesting content (at least 1 char)
                    (col + 1).min(line_text.len())
                };
                let width = end_col.saturating_sub(col).max(1);
                out.push_str(&format!("{} | {}{}\n", padding, " ".repeat(col), "^".repeat(width)));
            }
        }
    }

    // Secondary labels
    for label in &diag.labels {
        if let Some(src) = source {
            if label.span.start.line > 0 {
                if let Some(line_text) = get_source_line(src, label.span.start.line) {
                    out.push_str(&format!("  {}:{} | {}\n", label.span.start.line, label.span.start.col, line_text));
                    out.push_str(&format!("  = {}\n", label.message));
                }
            }
        }
    }

    // Notes
    for note in &diag.notes {
        out.push_str(&format!("  = note: {}\n", note));
    }

    out
}

/// Extract a 1-indexed source line.
fn get_source_line(source: &str, line: u32) -> Option<&str> {
    source.lines().nth((line as usize).checked_sub(1)?)
}

// ── JSON renderer (hand-written, no serde) ──────────────

/// Render a diagnostic as a JSON object.
///
/// ```json
/// {"code":"type_mismatch","severity":"error","message":"expected Int, found Bool",
///  "span":{"line":3,"col":5,"end_line":3,"end_col":9},"notes":["expected: Int","found: Bool"]}
/// ```
pub fn render_json(diag: &Diagnostic) -> String {
    let mut out = String::from("{");
    out.push_str(&format!("\"code\":{},", json_str(diag.code)));
    out.push_str(&format!("\"severity\":{},", json_str(&diag.severity.to_string())));
    out.push_str(&format!("\"message\":{}", json_str(&diag.message)));

    if let Some(span) = &diag.span {
        out.push_str(&format!(
            ",\"span\":{{\"line\":{},\"col\":{},\"end_line\":{},\"end_col\":{}}}",
            span.start.line, span.start.col, span.end.line, span.end.col
        ));
    }

    if !diag.notes.is_empty() {
        out.push_str(",\"notes\":[");
        for (i, note) in diag.notes.iter().enumerate() {
            if i > 0 { out.push(','); }
            out.push_str(&json_str(note));
        }
        out.push(']');
    }

    out.push('}');
    out
}

/// Escape a string for JSON output.
fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < '\x20' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// ── Error codes ─────────────────────────────────────────
// Stable string identifiers for programmatic consumption.

pub mod codes {
    // Lexer
    pub const LEX_INVALID_NUMBER: &str = "invalid_number";
    pub const LEX_UNTERMINATED_STRING: &str = "unterminated_string";
    pub const LEX_UNKNOWN_ESCAPE: &str = "unknown_escape";
    pub const LEX_UNEXPECTED_CHAR: &str = "unexpected_char";

    // Parser
    pub const PARSE_UNEXPECTED_TOKEN: &str = "unexpected_token";
    pub const PARSE_EXPECTED_EXPR: &str = "expected_expression";

    // Type checker
    pub const TYPE_MISMATCH: &str = "type_mismatch";
    pub const TYPE_INFINITE: &str = "infinite_type";
    pub const TYPE_UNBOUND_VAR: &str = "unbound_variable";
    pub const TYPE_UNBOUND_TYPE: &str = "unbound_type";
    pub const TYPE_ARITY: &str = "arity_mismatch";
    pub const TYPE_PATTERN: &str = "pattern_mismatch";
    pub const TYPE_OTHER: &str = "type_error";

    // Runtime (eval)
    pub const EVAL_UNDEFINED: &str = "undefined_variable";
    pub const EVAL_NO_MATCH: &str = "no_match";
    pub const EVAL_TYPE: &str = "runtime_type_error";
    pub const EVAL_DIV_ZERO: &str = "division_by_zero";
    pub const EVAL_IO: &str = "io_error";

    // Linearity (data race prevention)
    pub const LINEAR_DUPLICATE: &str = "linear_duplicate";
    pub const LINEAR_UNUSED: &str = "linear_unused";

    // Codegen
    pub const COMPILE_ERROR: &str = "compile_error";
}

// ── Tests ───────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use synoema_lexer::Pos;

    #[test]
    fn diagnostic_display() {
        let d = Diagnostic::error(codes::TYPE_MISMATCH, "expected Int, found Bool")
            .with_span(Span::new(
                Pos { line: 3, col: 5, offset: 20 },
                Pos { line: 3, col: 9, offset: 24 },
            ));
        assert_eq!(d.to_string(), "error[type_mismatch] at 3:5: expected Int, found Bool");
    }

    #[test]
    fn diagnostic_display_no_span() {
        let d = Diagnostic::error(codes::EVAL_DIV_ZERO, "division by zero");
        assert_eq!(d.to_string(), "error[division_by_zero]: division by zero");
    }

    #[test]
    fn render_human_with_source() {
        let d = Diagnostic::error(codes::TYPE_MISMATCH, "expected Int, found Bool")
            .with_span(Span::new(
                Pos { line: 1, col: 14, offset: 13 },
                Pos { line: 1, col: 18, offset: 17 },
            ))
            .with_note("expected: Int")
            .with_note("found: Bool");

        let source = "main = 1 + true";
        let output = render_human(&d, Some(source));
        assert!(output.contains("error[type_mismatch] at 1:14: expected Int, found Bool"));
        assert!(output.contains("main = 1 + true"));
        assert!(output.contains("^^^^"));
        assert!(output.contains("note: expected: Int"));
        assert!(output.contains("note: found: Bool"));
    }

    #[test]
    fn render_json_basic() {
        let d = Diagnostic::error(codes::TYPE_MISMATCH, "expected Int, found Bool")
            .with_span(Span::new(
                Pos { line: 3, col: 5, offset: 20 },
                Pos { line: 3, col: 9, offset: 24 },
            ))
            .with_note("expected: Int");

        let json = render_json(&d);
        assert!(json.contains("\"code\":\"type_mismatch\""));
        assert!(json.contains("\"severity\":\"error\""));
        assert!(json.contains("\"line\":3"));
        assert!(json.contains("\"col\":5"));
        assert!(json.contains("\"notes\":[\"expected: Int\"]"));
    }

    #[test]
    fn json_escaping() {
        let d = Diagnostic::error("test", "quotes \"here\" and\nnewline");
        let json = render_json(&d);
        assert!(json.contains("\\\"here\\\""));
        assert!(json.contains("\\n"));
    }

    #[test]
    fn get_line_works() {
        let src = "line one\nline two\nline three";
        assert_eq!(get_source_line(src, 1), Some("line one"));
        assert_eq!(get_source_line(src, 2), Some("line two"));
        assert_eq!(get_source_line(src, 3), Some("line three"));
        assert_eq!(get_source_line(src, 4), None);
        assert_eq!(get_source_line(src, 0), None);
    }
}
