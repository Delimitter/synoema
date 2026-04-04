// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

//! Source-text-based code formatter for Synoema.
//!
//! Operates on raw source text (not AST) because the parser is lossy:
//! string interpolation, record punning, and regular comments are
//! desugared or stripped during parsing.

use std::path::Path;

/// Format Synoema source code. Returns formatted source or parse error message.
pub fn format_source(source: &str) -> Result<String, String> {
    let formatted = apply_rules(source);

    // Validate: parse the formatted output to check for syntax errors
    synoema_parser::parse(&formatted)
        .map_err(|e| format!("parse error: {}", e))?;

    Ok(formatted)
}

fn apply_rules(source: &str) -> String {
    // 1. Split into lines, apply per-line rules
    let lines: Vec<String> = source.lines().map(|line| {
        // Tab → 2 spaces
        let line = line.replace('\t', "  ");
        // Remove trailing whitespace
        line.trim_end().to_string()
    }).collect();

    // 2. Collapse consecutive blank lines (2+ → 1)
    let mut result_lines: Vec<&str> = Vec::with_capacity(lines.len());
    let mut prev_blank = false;
    for line in &lines {
        let is_blank = line.is_empty();
        if is_blank && prev_blank {
            continue; // skip consecutive blank
        }
        result_lines.push(line);
        prev_blank = is_blank;
    }

    // 3. Remove leading blank lines
    while result_lines.first().map_or(false, |l| l.is_empty()) {
        result_lines.remove(0);
    }

    // 4. Remove trailing blank lines
    while result_lines.last().map_or(false, |l| l.is_empty()) {
        result_lines.pop();
    }

    // 5. Join and ensure exactly 1 trailing newline
    if result_lines.is_empty() {
        return String::from("\n");
    }
    let mut result = result_lines.join("\n");
    result.push('\n');
    result
}

/// Format a single file. Returns `true` if file was already formatted (or was formatted successfully).
/// In check mode, returns `false` if file needs formatting (no modification made).
pub fn format_file(path: &Path, check: bool) -> Result<bool, String> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("Error reading '{}': {}", path.display(), e))?;

    let formatted = format_source(&source)?;

    if check {
        Ok(source == formatted)
    } else {
        if source != formatted {
            std::fs::write(path, &formatted)
                .map_err(|e| format!("Error writing '{}': {}", path.display(), e))?;
        }
        Ok(true)
    }
}

/// Format all `.sno` files in a directory recursively.
/// Returns `(total_files, changed_files)` in format mode, or `(total, unformatted)` in check mode.
pub fn format_directory(dir: &Path, check: bool) -> Result<(usize, usize), String> {
    let mut total = 0;
    let mut changed = 0;

    walk_sno_files(dir, &mut |path| {
        total += 1;
        match format_file(path, check) {
            Ok(true) => {} // already formatted or formatted successfully
            Ok(false) => {
                // check mode: file needs formatting
                changed += 1;
                eprintln!("  needs formatting: {}", path.display());
            }
            Err(e) => {
                eprintln!("  error: {}: {}", path.display(), e);
                changed += 1;
            }
        }
    })?;

    Ok((total, changed))
}

fn walk_sno_files(dir: &Path, cb: &mut dyn FnMut(&Path)) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Error reading directory '{}': {}", dir.display(), e))?;

    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();
    paths.sort();

    for path in paths {
        if path.is_dir() {
            walk_sno_files(&path, cb)?;
        } else if path.extension().map_or(false, |ext| ext == "sno") {
            cb(&path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idempotency_simple_function() {
        let src = "add x y = x + y\nmain = add 1 2\n";
        let out = format_source(src).unwrap();
        let out2 = format_source(&out).unwrap();
        assert_eq!(out, out2);
    }

    #[test]
    fn idempotency_pattern_matching() {
        let src = "fib 0 = 0\nfib 1 = 1\nfib n = fib (n - 1) + fib (n - 2)\n\nmain = fib 10\n";
        let out = format_source(src).unwrap();
        let out2 = format_source(&out).unwrap();
        assert_eq!(out, out2);
    }

    #[test]
    fn idempotency_adt() {
        let src = "Shape = Circle Int | Rect Int Int\n\narea (Circle r) = r * r\narea (Rect w h) = w * h\n\nmain = area (Circle 5)\n";
        let out = format_source(src).unwrap();
        let out2 = format_source(&out).unwrap();
        assert_eq!(out, out2);
    }

    #[test]
    fn preserves_line_comments() {
        let src = "-- This is a comment\nmain = 42\n";
        let out = format_source(src).unwrap();
        assert!(out.contains("-- This is a comment"));
    }

    #[test]
    fn preserves_doc_comments() {
        let src = "--- Documentation for main\nmain = 42\n";
        let out = format_source(src).unwrap();
        assert!(out.contains("--- Documentation for main"));
    }

    #[test]
    fn replaces_tabs() {
        let src = "add x y = x + y\nmain = add\t1\t2\n";
        let out = format_source(src).unwrap();
        assert!(!out.contains('\t'));
        assert!(out.contains("main = add  1  2"));
    }

    #[test]
    fn removes_trailing_whitespace() {
        let src = "main = 42   \n";
        let out = format_source(src).unwrap();
        assert_eq!(out, "main = 42\n");
    }

    #[test]
    fn collapses_blank_lines() {
        let src = "foo = 1\n\n\n\nbar = 2\n";
        let out = format_source(src).unwrap();
        assert_eq!(out, "foo = 1\n\nbar = 2\n");
    }

    #[test]
    fn ensures_final_newline() {
        let src = "main = 42";
        let out = format_source(src).unwrap();
        assert!(out.ends_with('\n'));
        assert_eq!(out, "main = 42\n");
    }

    #[test]
    fn removes_leading_blank_lines() {
        let src = "\n\nmain = 42\n";
        let out = format_source(src).unwrap();
        assert_eq!(out, "main = 42\n");
    }

    #[test]
    fn malformed_code_returns_error() {
        let src = "main = + + +";
        let result = format_source(src);
        assert!(result.is_err());
    }

    #[test]
    fn empty_source_handled() {
        // Empty file has no valid declarations — parser may reject
        // At minimum, formatter should not panic
        let _ = format_source("");
    }

    #[test]
    fn already_formatted_is_noop() {
        let src = "main = 42\n";
        let out = format_source(src).unwrap();
        assert_eq!(src, out);
    }
}
