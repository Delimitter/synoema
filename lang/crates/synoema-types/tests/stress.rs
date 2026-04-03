//! Type-checker load tests.
//!
//! Fast tests:
//!   cargo test -p synoema-types stress
//!
//! All including slow:
//!   cargo test -p synoema-types stress -- --include-ignored
//!
//! Show timing:
//!   cargo test -p synoema-types stress -- --nocapture

use std::time::Instant;

fn ok(src: &str) {
    synoema_types::typecheck(src)
        .unwrap_or_else(|e| panic!("Type check failed:\n{}\nError: {}", src, e));
}

fn err_expected(src: &str) -> String {
    synoema_types::typecheck(src)
        .expect_err(&format!("Expected type error for:\n{}", src))
        .to_string()
}

// ── Generators ───────────────────────────────────────────────────────────────

/// Generate a chain of n definitions: x0=0, x1=x0+1, ..., main=x(n-1).
fn gen_let_chain(n: usize) -> String {
    let mut lines = Vec::with_capacity(n + 1);
    lines.push("x0 = 0".to_string());
    for i in 1..n {
        lines.push(format!("x{} = x{} + 1", i, i - 1));
    }
    lines.push(format!("main = x{}", n - 1));
    lines.join("\n")
}

/// Generate a chain WITHOUT the `main` line (for T-6 type-error test).
fn gen_let_chain_no_main(n: usize) -> String {
    let mut lines = Vec::with_capacity(n);
    lines.push("x0 = 0".to_string());
    for i in 1..n {
        lines.push(format!("x{} = x{} + 1", i, i - 1));
    }
    lines.join("\n")
}

/// Generate a record with n fields: r = {f0=0, f1=1, ..., f(n-1)=n-1}.
fn gen_wide_record(n: usize) -> String {
    let fields = (0..n)
        .map(|i| format!("f{} = {}", i, i))
        .collect::<Vec<_>>()
        .join(", ");
    format!("r = {{{}}}\nmain = 0", fields)
}

/// Generate an ADT with n unit constructors.
fn gen_wide_adt(n: usize) -> String {
    let ctors = (0..n)
        .map(|i| format!("C{}", i))
        .collect::<Vec<_>>()
        .join(" | ");
    format!("Color = {}\nmain = 0", ctors)
}

/// Generate an ADT with n constructors plus n pattern-match equations.
fn gen_wide_adt_with_match(n: usize) -> String {
    let ctors = (0..n)
        .map(|i| format!("C{}", i))
        .collect::<Vec<_>>()
        .join(" | ");
    let arms = (0..n)
        .map(|i| format!("toInt C{} = {}", i, i))
        .collect::<Vec<_>>()
        .join("\n");
    format!("Color = {}\n{}\nmain = 0", ctors, arms)
}

// ── T-1: Let-polymorphism scale ───────────────────────────────────────────────

#[test]
fn t1_let_chain_100() {
    let src = gen_let_chain(100);
    let start = Instant::now();
    ok(&src);
    let elapsed = start.elapsed();
    println!("T-1: 100-chain in {:?}", elapsed);
}

#[test]
fn t1_let_chain_1000() {
    let src = gen_let_chain(1000);
    let start = Instant::now();
    ok(&src);
    let elapsed = start.elapsed();
    println!("T-1: 1000-chain in {:?}  (release threshold: 1 s)", elapsed);
    #[cfg(not(debug_assertions))]
    assert!(elapsed.as_secs() < 1, "1000-chain took {:?}", elapsed);
}

#[test]
#[ignore]
fn t1_let_chain_5000() {
    let src = gen_let_chain(5_000);
    let start = Instant::now();
    ok(&src);
    let elapsed = start.elapsed();
    println!("T-1: 5000-chain in {:?}", elapsed);
}

// ── T-3: Wide record — row polymorphism ──────────────────────────────────────

#[test]
fn t3_wide_record_10_fields() {
    ok(&gen_wide_record(10));
    println!("T-3: 10-field record — OK");
}

#[test]
fn t3_wide_record_50_fields() {
    let src = gen_wide_record(50);
    let start = Instant::now();
    ok(&src);
    let elapsed = start.elapsed();
    println!("T-3: 50-field record in {:?}", elapsed);
}

#[test]
fn t3_wide_record_100_fields() {
    let src = gen_wide_record(100);
    let start = Instant::now();
    ok(&src);
    let elapsed = start.elapsed();
    println!("T-3: 100-field record in {:?}  (release threshold: 100 ms)", elapsed);
    #[cfg(not(debug_assertions))]
    assert!(elapsed.as_millis() < 100, "100-field record took {:?}", elapsed);
}

// ── T-5: Many independent recursive functions ────────────────────────────────

#[test]
fn t5_independent_recursive_fns_20() {
    // 20 independent self-recursive functions — stresses type-var allocation
    // and generalisation. (The type checker processes definitions sequentially,
    // so forward/mutual references are not supported at top level.)
    let mut lines: Vec<String> = Vec::new();
    for i in 0..20 {
        lines.push(format!(
            "countdown{} n = ? n == 0 -> 0 : countdown{} (n - 1)",
            i, i
        ));
    }
    lines.push("main = 0".to_string());
    let src = lines.join("\n");

    let start = Instant::now();
    ok(&src);
    let elapsed = start.elapsed();
    println!("T-5: 20 independent recursive functions in {:?}", elapsed);
}

// ── T-6: Type error — diagnostic speed ───────────────────────────────────────

#[test]
fn t6_type_error_detected_fast_in_large_program() {
    // 499 correct definitions + 1 type error (Int + Bool) at position 500.
    let base = gen_let_chain_no_main(499); // x0..x498
    let src = format!("{}\nx499 = x498 + true\nmain = x499", base);

    let start = Instant::now();
    let e = err_expected(&src);
    let elapsed = start.elapsed();
    println!(
        "T-6: type error in 500-def program detected in {:?}  (release threshold: 200 ms)\n  error: {}",
        elapsed,
        e.lines().next().unwrap_or("")
    );
    // Threshold only enforced in release mode (debug is ~10× slower).
    #[cfg(not(debug_assertions))]
    assert!(elapsed.as_millis() < 200, "Type error detection took {:?}", elapsed);
}

// ── T-7: ADT with many constructors ──────────────────────────────────────────

#[test]
fn t7_adt_100_constructors() {
    let src = gen_wide_adt(100);
    let start = Instant::now();
    ok(&src);
    let elapsed = start.elapsed();
    println!("T-7: 100 constructors registered in {:?}  (threshold: 100 ms)", elapsed);
    assert!(elapsed.as_millis() < 100, "100-ctor ADT took {:?}", elapsed);
}

// ── T-8a: Wide ADT case match ─────────────────────────────────────────────────

#[test]
fn t8a_wide_adt_100_match_arms() {
    let src = gen_wide_adt_with_match(100);
    let start = Instant::now();
    ok(&src);
    let elapsed = start.elapsed();
    println!("T-8a: 100-arm ADT match type-checked in {:?}  (release threshold: 50 ms)", elapsed);
    #[cfg(not(debug_assertions))]
    assert!(elapsed.as_millis() < 50, "100-arm match took {:?}", elapsed);
}

// ── Throughput bench ─────────────────────────────────────────────────────────

#[test]
#[ignore]
fn bench_typecheck_100_chain_throughput() {
    let src = gen_let_chain(100);
    let n = 500u32;

    // Warmup
    for _ in 0..10 {
        let _ = synoema_types::typecheck(&src);
    }

    let start = Instant::now();
    for _ in 0..n {
        synoema_types::typecheck(&src).unwrap();
    }
    let avg = start.elapsed() / n;
    println!("BENCH typecheck_100chain: {:?} avg/run", avg);
}
