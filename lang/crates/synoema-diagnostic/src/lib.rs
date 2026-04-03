// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

//! # synoema-diagnostic
//! Structured error reporting for the Synoema programming language.
//!
//! Provides a unified `Diagnostic` type with renderers for human-readable
//! and JSON output, optimized for LLM consumption in the generate → check → fix loop.

use synoema_lexer::Span;
use std::fmt;

// ── Fixability ──────────────────────────────────────────

/// How difficult an error is to fix — guides LLM retry strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fixability {
    /// Typo, missing delimiter, wrong symbol
    Trivial,
    /// Add argument, fix type, add pattern
    Easy,
    /// Restructure code, redesign types
    Medium,
    /// Rethink algorithm, infinite type
    Hard,
}

impl fmt::Display for Fixability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fixability::Trivial => write!(f, "trivial"),
            Fixability::Easy => write!(f, "easy"),
            Fixability::Medium => write!(f, "medium"),
            Fixability::Hard => write!(f, "hard"),
        }
    }
}

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
    /// Actionable fix instruction for LLM feedback loops.
    pub llm_hint: Option<String>,
    /// How difficult the error is to fix.
    pub fixability: Option<Fixability>,
    /// Suggested alternative syntax (e.g. "if x then y" → "? x -> y : z").
    pub did_you_mean: Option<String>,
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
            llm_hint: None,
            fixability: None,
            did_you_mean: None,
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
            llm_hint: None,
            fixability: None,
            did_you_mean: None,
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

    /// Attach an LLM hint (actionable fix instruction).
    pub fn with_llm_hint(mut self, hint: impl Into<String>) -> Self {
        self.llm_hint = Some(hint.into());
        self
    }

    /// Attach fixability level.
    pub fn with_fixability(mut self, f: Fixability) -> Self {
        self.fixability = Some(f);
        self
    }

    /// Attach a did-you-mean suggestion.
    pub fn with_did_you_mean(mut self, suggestion: impl Into<String>) -> Self {
        self.did_you_mean = Some(suggestion.into());
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

    // LLM hint
    if let Some(ref hint) = diag.llm_hint {
        out.push_str(&format!("  = hint: {}\n", hint));
    }

    // Did-you-mean
    if let Some(ref dym) = diag.did_you_mean {
        out.push_str(&format!("  = did you mean: {}\n", dym));
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

    if let Some(ref hint) = diag.llm_hint {
        out.push_str(&format!(",\"llm_hint\":{}", json_str(hint)));
    }

    if let Some(ref f) = diag.fixability {
        out.push_str(&format!(",\"fixability\":{}", json_str(&f.to_string())));
    }

    if let Some(ref dym) = diag.did_you_mean {
        out.push_str(&format!(",\"did_you_mean\":{}", json_str(dym)));
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

    // Imports
    pub const IMPORT_CYCLE: &str = "import_cycle";
    pub const IMPORT_NOT_FOUND: &str = "import_not_found";

    // Indentation
    pub const PARSE_INDENTATION: &str = "indentation";

    // Codegen
    pub const COMPILE_ERROR: &str = "compile_error";
}

// ── LLM Error Enrichment ───────────────────────────────
// Adds llm_hint, fixability, and did_you_mean to diagnostics
// for the top-10 most frequent LLM errors.

/// Enrich a diagnostic with LLM-actionable metadata.
/// Call this before rendering to add hints, fixability, and did-you-mean.
pub fn enrich_diagnostic(diag: &mut Diagnostic) {
    // Skip if already enriched
    if diag.llm_hint.is_some() {
        return;
    }

    match diag.code {
        codes::TYPE_MISMATCH => {
            diag.fixability = Some(Fixability::Trivial);
            let (expected, found) = extract_expected_found(&diag.notes);
            diag.llm_hint = Some(format!(
                "Change the expression to produce {} instead of {}. \
                 Common fixes: type conversion, different operator, or fix the literal value.",
                expected, found
            ));
        }
        codes::TYPE_ARITY => {
            diag.fixability = Some(Fixability::Trivial);
            diag.llm_hint = Some(
                "Wrong number of arguments. Check the function signature and add/remove arguments.".into()
            );
        }
        codes::TYPE_UNBOUND_VAR => {
            diag.fixability = Some(Fixability::Easy);
            let name = extract_name_from_message(&diag.message);
            diag.llm_hint = Some(format!(
                "Variable '{}' is not defined. Check spelling, add it as a parameter, or define it in a where-block.",
                name
            ));
        }
        codes::TYPE_INFINITE => {
            diag.fixability = Some(Fixability::Hard);
            diag.llm_hint = Some(
                "Infinite type detected — the type refers to itself. \
                 Restructure to break the cycle, e.g. wrap in an ADT constructor.".into()
            );
        }
        codes::TYPE_PATTERN => {
            diag.fixability = Some(Fixability::Easy);
            diag.llm_hint = Some(
                "Pattern does not match the expected type. Check constructor names and arity.".into()
            );
        }
        codes::PARSE_UNEXPECTED_TOKEN => {
            diag.fixability = Some(Fixability::Trivial);
            // Apply did-you-mean rules for common LLM syntax mistakes
            apply_syntax_did_you_mean(diag);
            if diag.llm_hint.is_none() {
                diag.llm_hint = Some(
                    "Unexpected token. Check for missing operators, extra punctuation, \
                     or unsupported syntax (no if/then/else, no commas in lists, no return).".into()
                );
            }
        }
        codes::PARSE_EXPECTED_EXPR => {
            diag.fixability = Some(Fixability::Trivial);
            diag.llm_hint = Some(
                "Expected an expression. Every construct in Synoema is an expression — \
                 there are no statements.".into()
            );
        }
        codes::LEX_UNTERMINATED_STRING => {
            diag.fixability = Some(Fixability::Trivial);
            diag.llm_hint = Some(
                "String literal is missing the closing quote. Add a matching '\"' at the end.".into()
            );
        }
        codes::EVAL_NO_MATCH => {
            diag.fixability = Some(Fixability::Easy);
            diag.llm_hint = Some(
                "No pattern matched the input value. Add a catch-all pattern or handle the missing case.".into()
            );
        }
        codes::EVAL_DIV_ZERO => {
            diag.fixability = Some(Fixability::Trivial);
            diag.llm_hint = Some(
                "Division by zero. Guard with a conditional: ? divisor == 0 -> default : x / divisor".into()
            );
        }
        codes::LINEAR_UNUSED => {
            diag.fixability = Some(Fixability::Easy);
            diag.llm_hint = Some(
                "Linear variable declared but not used. Use it in the body or remove it from parameters.".into()
            );
        }
        codes::LINEAR_DUPLICATE => {
            diag.fixability = Some(Fixability::Easy);
            diag.llm_hint = Some(
                "Linear variable used more than once. Use it exactly once, or copy the value first.".into()
            );
        }
        codes::PARSE_INDENTATION => {
            diag.fixability = Some(Fixability::Easy);
            if diag.llm_hint.is_none() {
                diag.llm_hint = Some(
                    "Synoema uses the offside rule (like Haskell/Python). \
                     Indent the body of a definition further than its name. \
                     Use consistent 2-space indentation.".into()
                );
            }
        }
        _ => {}
    }
}

/// Apply did-you-mean rules for common LLM syntax mistakes.
fn apply_syntax_did_you_mean(diag: &mut Diagnostic) {
    let msg = diag.message.to_lowercase();

    // if/then/else → ternary
    if msg.contains("if") || msg.contains("then") || msg.contains("else") {
        diag.did_you_mean = Some("? condition -> then_expr : else_expr".into());
        diag.llm_hint = Some(
            "Synoema has no if/then/else. Use ternary: ? cond -> x : y".into()
        );
        return;
    }

    // Commas in lists → spaces
    if msg.contains(",") || msg.contains("comma") {
        diag.did_you_mean = Some("[1 2 3] (space-separated, no commas)".into());
        diag.llm_hint = Some(
            "Lists use spaces, not commas: [1 2 3] not [1, 2, 3]".into()
        );
        return;
    }

    // return → expression-based
    if msg.contains("return") {
        diag.did_you_mean = Some("just write the expression (Synoema is expression-based)".into());
        diag.llm_hint = Some(
            "Synoema has no 'return'. The last expression is the result.".into()
        );
        return;
    }

    // -> without backslash (lambda)
    if msg.contains("->") && !msg.contains("\\") {
        diag.did_you_mean = Some("\\x -> body (lambda needs backslash)".into());
        diag.llm_hint = Some(
            "Lambda syntax requires backslash: \\x -> x + 1".into()
        );
    }
}

/// Extract expected/found types from diagnostic notes.
fn extract_expected_found(notes: &[String]) -> (String, String) {
    let mut expected = "the expected type".to_string();
    let mut found = "the actual type".to_string();
    for note in notes {
        if let Some(rest) = note.strip_prefix("expected: ") {
            expected = rest.to_string();
        } else if let Some(rest) = note.strip_prefix("found: ") {
            found = rest.to_string();
        }
    }
    (expected, found)
}

/// Extract a variable/function name from an error message.
fn extract_name_from_message(msg: &str) -> String {
    // Try to find a quoted name like 'foo' or `foo`
    if let Some(start) = msg.find('\'') {
        if let Some(end) = msg[start + 1..].find('\'') {
            return msg[start + 1..start + 1 + end].to_string();
        }
    }
    if let Some(start) = msg.find('`') {
        if let Some(end) = msg[start + 1..].find('`') {
            return msg[start + 1..start + 1 + end].to_string();
        }
    }
    // Fallback: try "Unbound variable: name" pattern
    if let Some(rest) = msg.strip_prefix("Unbound variable: ") {
        return rest.trim().to_string();
    }
    "<unknown>".to_string()
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

    // ── Enrichment tests ──────────────────────────────────

    #[test]
    fn enrich_type_mismatch() {
        let mut d = Diagnostic::error(codes::TYPE_MISMATCH, "expected Int, found Bool")
            .with_note("expected: Int")
            .with_note("found: Bool");
        enrich_diagnostic(&mut d);
        assert_eq!(d.fixability, Some(Fixability::Trivial));
        assert!(d.llm_hint.as_ref().unwrap().contains("Int"));
        assert!(d.llm_hint.as_ref().unwrap().contains("Bool"));
    }

    #[test]
    fn enrich_type_arity() {
        let mut d = Diagnostic::error(codes::TYPE_ARITY, "wrong number of arguments");
        enrich_diagnostic(&mut d);
        assert_eq!(d.fixability, Some(Fixability::Trivial));
        assert!(d.llm_hint.is_some());
    }

    #[test]
    fn enrich_unbound_var() {
        let mut d = Diagnostic::error(codes::TYPE_UNBOUND_VAR, "Unbound variable: foo");
        enrich_diagnostic(&mut d);
        assert_eq!(d.fixability, Some(Fixability::Easy));
        assert!(d.llm_hint.as_ref().unwrap().contains("foo"));
    }

    #[test]
    fn enrich_infinite_type() {
        let mut d = Diagnostic::error(codes::TYPE_INFINITE, "infinite type");
        enrich_diagnostic(&mut d);
        assert_eq!(d.fixability, Some(Fixability::Hard));
    }

    #[test]
    fn enrich_pattern_mismatch() {
        let mut d = Diagnostic::error(codes::TYPE_PATTERN, "pattern mismatch");
        enrich_diagnostic(&mut d);
        assert_eq!(d.fixability, Some(Fixability::Easy));
    }

    #[test]
    fn enrich_unexpected_token() {
        let mut d = Diagnostic::error(codes::PARSE_UNEXPECTED_TOKEN, "unexpected token");
        enrich_diagnostic(&mut d);
        assert_eq!(d.fixability, Some(Fixability::Trivial));
        assert!(d.llm_hint.is_some());
    }

    #[test]
    fn enrich_unterminated_string() {
        let mut d = Diagnostic::error(codes::LEX_UNTERMINATED_STRING, "unterminated string");
        enrich_diagnostic(&mut d);
        assert_eq!(d.fixability, Some(Fixability::Trivial));
        assert!(d.llm_hint.as_ref().unwrap().contains("closing quote"));
    }

    #[test]
    fn enrich_no_match() {
        let mut d = Diagnostic::error(codes::EVAL_NO_MATCH, "no pattern matched");
        enrich_diagnostic(&mut d);
        assert_eq!(d.fixability, Some(Fixability::Easy));
        assert!(d.llm_hint.as_ref().unwrap().contains("catch-all"));
    }

    #[test]
    fn enrich_div_zero() {
        let mut d = Diagnostic::error(codes::EVAL_DIV_ZERO, "division by zero");
        enrich_diagnostic(&mut d);
        assert_eq!(d.fixability, Some(Fixability::Trivial));
    }

    #[test]
    fn enrich_linear_unused() {
        let mut d = Diagnostic::error(codes::LINEAR_UNUSED, "unused linear var");
        enrich_diagnostic(&mut d);
        assert_eq!(d.fixability, Some(Fixability::Easy));
    }

    #[test]
    fn enrich_linear_duplicate() {
        let mut d = Diagnostic::error(codes::LINEAR_DUPLICATE, "duplicate linear var");
        enrich_diagnostic(&mut d);
        assert_eq!(d.fixability, Some(Fixability::Easy));
    }

    #[test]
    fn enrich_indentation() {
        let mut d = Diagnostic::error(codes::PARSE_INDENTATION, "indentation error");
        enrich_diagnostic(&mut d);
        assert_eq!(d.fixability, Some(Fixability::Easy));
        assert!(d.llm_hint.as_ref().unwrap().contains("offside"));
    }

    // ── Did-you-mean tests ─────────────────────────────────

    #[test]
    fn did_you_mean_if_then_else() {
        let mut d = Diagnostic::error(codes::PARSE_UNEXPECTED_TOKEN, "unexpected if token");
        enrich_diagnostic(&mut d);
        assert!(d.did_you_mean.as_ref().unwrap().contains("?"));
        assert!(d.llm_hint.as_ref().unwrap().contains("ternary"));
    }

    #[test]
    fn did_you_mean_comma() {
        let mut d = Diagnostic::error(codes::PARSE_UNEXPECTED_TOKEN, "unexpected ','");
        enrich_diagnostic(&mut d);
        assert!(d.did_you_mean.as_ref().unwrap().contains("space-separated"));
    }

    #[test]
    fn did_you_mean_return() {
        let mut d = Diagnostic::error(codes::PARSE_UNEXPECTED_TOKEN, "unexpected return");
        enrich_diagnostic(&mut d);
        assert!(d.did_you_mean.as_ref().unwrap().contains("expression"));
    }

    // ── JSON output with enrichment fields ─────────────────

    #[test]
    fn json_includes_llm_fields() {
        let d = Diagnostic::error(codes::TYPE_MISMATCH, "expected Int, found Bool")
            .with_llm_hint("Fix the type")
            .with_fixability(Fixability::Trivial)
            .with_did_you_mean("use Int instead");
        let json = render_json(&d);
        assert!(json.contains("\"llm_hint\":\"Fix the type\""));
        assert!(json.contains("\"fixability\":\"trivial\""));
        assert!(json.contains("\"did_you_mean\":\"use Int instead\""));
    }

    #[test]
    fn json_omits_null_llm_fields() {
        let d = Diagnostic::error(codes::TYPE_MISMATCH, "expected Int, found Bool");
        let json = render_json(&d);
        assert!(!json.contains("llm_hint"));
        assert!(!json.contains("fixability"));
        assert!(!json.contains("did_you_mean"));
    }

    // ── Human renderer with enrichment ─────────────────────

    #[test]
    fn human_shows_hint() {
        let d = Diagnostic::error(codes::TYPE_MISMATCH, "expected Int, found Bool")
            .with_llm_hint("Fix this")
            .with_did_you_mean("? cond -> x : y");
        let output = render_human(&d, None);
        assert!(output.contains("hint: Fix this"));
        assert!(output.contains("did you mean: ? cond -> x : y"));
    }

    #[test]
    fn enrich_skips_if_already_enriched() {
        let mut d = Diagnostic::error(codes::TYPE_MISMATCH, "expected Int, found Bool")
            .with_llm_hint("Custom hint");
        enrich_diagnostic(&mut d);
        assert_eq!(d.llm_hint.as_ref().unwrap(), "Custom hint");
    }

    // ── Fixability display ─────────────────────────────────

    #[test]
    fn fixability_display() {
        assert_eq!(format!("{}", Fixability::Trivial), "trivial");
        assert_eq!(format!("{}", Fixability::Easy), "easy");
        assert_eq!(format!("{}", Fixability::Medium), "medium");
        assert_eq!(format!("{}", Fixability::Hard), "hard");
    }

    // ── Builder methods ────────────────────────────────────

    #[test]
    fn builder_chain() {
        let d = Diagnostic::error(codes::TYPE_MISMATCH, "test")
            .with_llm_hint("hint")
            .with_fixability(Fixability::Easy)
            .with_did_you_mean("suggestion");
        assert_eq!(d.llm_hint.as_ref().unwrap(), "hint");
        assert_eq!(d.fixability, Some(Fixability::Easy));
        assert_eq!(d.did_you_mean.as_ref().unwrap(), "suggestion");
    }

    // ── Helper tests ───────────────────────────────────────

    #[test]
    fn extract_expected_found_works() {
        let notes = vec![
            "expected: Int".to_string(),
            "found: String".to_string(),
        ];
        let (e, f) = extract_expected_found(&notes);
        assert_eq!(e, "Int");
        assert_eq!(f, "String");
    }

    #[test]
    fn extract_name_from_message_quoted() {
        assert_eq!(extract_name_from_message("Unbound variable 'foo'"), "foo");
        assert_eq!(extract_name_from_message("Unbound variable: bar"), "bar");
        assert_eq!(extract_name_from_message("something else"), "<unknown>");
    }
}
