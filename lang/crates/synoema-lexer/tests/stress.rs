// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025-present Synoema Contributors

//! Lexer load tests.
//!
//! Fast tests (run normally):
//!   cargo test -p synoema-lexer stress
//!
//! All tests including slow:
//!   cargo test -p synoema-lexer stress -- --include-ignored
//!
//! Show timing output:
//!   cargo test -p synoema-lexer stress -- --nocapture

use std::time::Instant;
use synoema_lexer::{lex, lex_tokens};

// ── L-1: Linear scale ────────────────────────────────────────────────────────

/// L-1: ~10K tokens, verify basic O(n) behaviour.
#[test]
fn l1_linear_10k_tokens() {
    let src = "x + ".repeat(3_333); // ~10K tokens (each "x + " → 2 tokens)
    let start = Instant::now();
    let toks = lex(&src).expect("lex failed");
    let elapsed = start.elapsed();
    println!("L-1: {} tokens in {:?}", toks.len(), elapsed);
    assert!(elapsed.as_millis() < 200, "10K tokens took {:?}", elapsed);
}

/// L-1: ~100K tokens, threshold < 50 ms.
#[test]
fn l1_linear_100k_tokens() {
    let src = "x + ".repeat(33_333); // ~100K tokens
    let start = Instant::now();
    let toks = lex(&src).expect("lex failed");
    let elapsed = start.elapsed();
    println!("L-1: {} tokens in {:?}  (release threshold: 50 ms)", toks.len(), elapsed);
    #[cfg(not(debug_assertions))]
    assert!(elapsed.as_millis() < 50, "100K tokens took {:?}, expected < 50 ms", elapsed);
}

/// L-1: ~1M tokens — slow, run with --include-ignored.
#[test]
#[ignore]
fn l1_linear_1m_tokens() {
    let src = "x + ".repeat(333_333);
    let start = Instant::now();
    let toks = lex(&src).expect("lex failed");
    let elapsed = start.elapsed();
    println!("L-1: {} tokens in {:?}", toks.len(), elapsed);
    // Just verify no crash; O(n) means < 500 ms expected.
}

// ── L-2: Long identifiers ────────────────────────────────────────────────────

/// L-2: 100 identifiers each 10K chars — no O(n²) string building.
#[test]
fn l2_long_identifiers() {
    let long_id = "x".repeat(10_000);
    let src = std::iter::repeat(format!("{} + ", long_id))
        .take(100)
        .collect::<String>();
    let start = Instant::now();
    lex(&src).expect("lex failed");
    let elapsed = start.elapsed();
    println!("L-2: 100 × 10K-char identifiers in {:?}  (threshold: 100 ms)", elapsed);
    assert!(elapsed.as_millis() < 100, "Long identifiers took {:?}", elapsed);
}

// ── L-3: Deep indentation (offside rule) ─────────────────────────────────────

/// L-3: 100 levels of indentation — INDENT/DEDENT stack must not overflow.
#[test]
fn l3_deep_indent_100_levels() {
    let mut src = String::new();
    for i in 0..100 {
        let indent = "  ".repeat(i);
        src.push_str(&format!("{}let f{} x =\n", indent, i));
    }
    src.push_str(&"  ".repeat(100));
    src.push_str("42\n");

    let result = lex(&src);
    assert!(result.is_ok(), "100 indent levels failed: {:?}", result.err());
    println!("L-3: 100 indent levels — OK");
}

/// L-3: 1000 levels — stress the INDENT/DEDENT stack.
#[test]
#[ignore]
fn l3_deep_indent_1000_levels() {
    let mut src = String::new();
    for i in 0..1000 {
        let indent = "  ".repeat(i);
        src.push_str(&format!("{}let f{} x =\n", indent, i));
    }
    src.push_str(&"  ".repeat(1000));
    src.push_str("42\n");

    let result = lex(&src);
    assert!(result.is_ok(), "1000 indent levels failed: {:?}", result.err());
    println!("L-3: 1000 indent levels — OK");
}

// ── L-4: Long strings with escape sequences ───────────────────────────────────

/// L-4: 10K-char string built from \" and \\ pairs — no O(n²) char push.
#[test]
fn l4_long_escaped_string() {
    // Each r#"\""# is two visible chars; combined gives: \"\"...\"
    let inner = r#"\"\""#.repeat(2_500); // 10K visible chars in source
    let src = format!("x = \"{}\"", inner);
    let start = Instant::now();
    let result = lex(&src);
    let elapsed = start.elapsed();
    println!("L-4: 10K-char escaped string in {:?}  (threshold: 50 ms)", elapsed);
    assert!(result.is_ok(), "Escaped string failed: {:?}", result.err());
    assert!(elapsed.as_millis() < 50, "Escaped string took {:?}", elapsed);
}

// ── L-5: Boundary numbers ─────────────────────────────────────────────────────

/// L-5: Numeric literals at i64/f64 boundaries — no panic, graceful result.
#[test]
fn l5_boundary_numbers() {
    // i64::MAX
    let src = "x = 9223372036854775807";
    let r = lex_tokens(src);
    assert!(r.is_ok(), "i64::MAX failed: {:?}", r.err());

    // One beyond i64::MAX — should return Err or parse as float, never panic.
    let r2 = lex("x = 9223372036854775808");
    println!("L-5: i64::MAX+1 → {}", if r2.is_ok() { "Ok" } else { "Err" });

    // f64::MAX as scientific notation
    let r3 = lex("x = 1.7976931348623157e308");
    println!("L-5: f64::MAX → {}", if r3.is_ok() { "Ok" } else { "Err" });

    // Zero
    assert!(lex_tokens("x = 0").is_ok());
    assert!(lex_tokens("x = 0.0").is_ok());

    println!("L-5: boundary numbers — no panics");
}

// ── L-7: Unicode / UTF-8 ─────────────────────────────────────────────────────

/// L-7: Unicode string literal — cyrillic, CJK, emoji.
#[test]
fn l7_unicode_string_literal() {
    let src = r#"x = "Привет мир 中文 日本語 🎉🦀""#;
    let result = lex(src);
    assert!(result.is_ok(), "Unicode string literal failed: {:?}", result.err());
    println!("L-7: Unicode string literal — OK");
}

/// L-7: ~10K-char Unicode string — O(n) byte scan.
#[test]
fn l7_unicode_string_10k_chars() {
    let content = "Привет ".repeat(1_428); // ~10K chars (7 bytes × 1428 = 9996 chars)
    let src = format!("x = \"{}\"", content);
    let start = Instant::now();
    let result = lex(&src);
    let elapsed = start.elapsed();
    println!("L-7: ~10K Unicode chars in {:?}  (threshold: 10 ms)", elapsed);
    assert!(result.is_ok(), "Large Unicode string failed: {:?}", result.err());
    assert!(elapsed.as_millis() < 10, "Unicode lexing took {:?}", elapsed);
}

// ── EE-7b: Adversarial / robustness ──────────────────────────────────────────

/// EE-7b: Various invalid/garbage inputs — must return Err, never panic.
#[test]
fn ee7b_adversarial_no_panic() {
    let inputs: &[&str] = &[
        "@#$%^&*",
        "))))(((([[[[",
        "→ ← ↑ ↓ ∞",
        "🦀🦀🦀",
        "let let let let",
        "= = = = =",
        "999999999999999999999999999999", // overflow integer
    ];
    for input in inputs {
        let result = lex(input);
        // Either Ok (tokenised as something) or Err — must never panic.
        let preview: String = input.chars().take(30).collect();
        println!("EE-7b: {:?} → {}", preview, if result.is_ok() { "Ok" } else { "Err" });
    }
    println!("EE-7b: adversarial inputs — no panics");
}

/// EE-7b: 100KB of repeated garbage bytes — no panic.
#[test]
fn ee7b_large_garbage_input() {
    let garbage = "@#!$%".repeat(20_000); // 100KB
    let result = lex(&garbage);
    println!("EE-7b: 100KB garbage → {}", if result.is_ok() { "Ok" } else { "Err (expected)" });
    // Either result is fine — the important thing is no panic/SIGSEGV.
}

// ── Throughput bench (manual, ignore by default) ─────────────────────────────

/// Bench: 100 iterations of lexing ~100K tokens, prints avg time.
#[test]
#[ignore]
fn bench_lex_100k_throughput() {
    let src = "x + ".repeat(33_333);
    let n = 100u32;

    // Warmup
    for _ in 0..5 {
        let _ = lex(&src);
    }

    let start = Instant::now();
    for _ in 0..n {
        let _ = lex(&src).unwrap();
    }
    let avg = start.elapsed() / n;
    println!("BENCH lex_100k: {:?} avg/run", avg);
}
