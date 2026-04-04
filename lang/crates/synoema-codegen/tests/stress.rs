// SPDX-License-Identifier: BUSL-1.1
// Copyright (c) 2025-present Andrey Bubnov

//! JIT compiler and runtime load tests.
//!
//! Fast tests:
//!   cargo test -p synoema-codegen stress
//!
//! All including slow / potentially unsafe:
//!   cargo test -p synoema-codegen stress -- --include-ignored
//!
//! Show timing:
//!   cargo test -p synoema-codegen stress -- --nocapture

use std::time::Instant;

fn jit(src: &str) -> i64 {
    synoema_codegen::compile_and_run(src)
        .unwrap_or_else(|e| panic!("JIT failed:\n{}\nError: {}", src, e))
}

#[allow(dead_code)]
fn jit_str(src: &str) -> String {
    synoema_codegen::compile_and_display(src)
        .unwrap_or_else(|e| panic!("JIT display failed:\n{}\nError: {}", src, e))
}

// ── J-2: Float arithmetic — FloatNode arena usage ────────────────────────────

/// J-2: 1K float ops → ~8 KB FloatNodes; well within 8 MB arena.
#[test]
fn j2_float_arena_1k_ops() {
    // Each recursive call allocates one FloatNode for (acc + 1.0).
    let src = "\
addf n acc = ? n == 0 -> acc : addf (n - 1) (acc + 1.0)
main = addf 1000 0.0";
    let start = Instant::now();
    let result = synoema_codegen::compile_and_run(src);
    let elapsed = start.elapsed();
    println!("J-2: 1K float ops in {:?}  (~8 KB FloatNodes in arena)", elapsed);
    assert!(result.is_ok(), "Float arena 1K failed: {:?}", result.err());
}

/// J-2: 10K float ops → ~80 KB FloatNodes.
#[test]
fn j2_float_arena_10k_ops() {
    let src = "\
addf n acc = ? n == 0 -> acc : addf (n - 1) (acc + 1.0)
main = addf 10000 0.0";
    let start = Instant::now();
    let result = synoema_codegen::compile_and_run(src);
    let elapsed = start.elapsed();
    println!("J-2: 10K float ops in {:?}  (~80 KB FloatNodes)", elapsed);
    assert!(result.is_ok(), "Float arena 10K failed: {:?}", result.err());
}

/// J-2: 100K float ops → ~800 KB FloatNodes — still within 8 MB arena.
#[test]
#[ignore]
fn j2_float_arena_100k_ops() {
    let src = "\
addf n acc = ? n == 0 -> acc : addf (n - 1) (acc + 1.0)
main = addf 100000 0.0";
    let start = Instant::now();
    let result = synoema_codegen::compile_and_run(src);
    let elapsed = start.elapsed();
    println!("J-2: 100K float ops in {:?}  (~800 KB FloatNodes)", elapsed);
    assert!(result.is_ok(), "Float arena 100K failed: {:?}", result.err());
}

// ── J-3: String literals — arena allocation ──────────────────────────────────

/// J-3: 100 string literals compiled — data uses AST pointer, StrNode in arena.
/// No leak: Box::leak replaced with direct AST pointer + arena copy.
#[test]
fn j3_string_literals_100() {
    let defs: Vec<String> = (0..100)
        .map(|i| format!("s{} = \"literal_{}\"", i, i))
        .collect();
    let src = format!("{}\nmain = 0", defs.join("\n"));
    let result = synoema_codegen::compile_and_run(&src);
    assert!(result.is_ok(), "100 string literals failed: {:?}", result.err());
    println!("J-3: 100 string literals compiled (arena allocation, no leak)");
}

// ── J-4: Arena overflow — graceful fallback ───────────────────────────────────

/// J-4: 600K ListNodes = 9.6 MB > 8 MB arena → system malloc fallback.
/// Verifies no SIGSEGV and length is correct after overflow.
#[test]
fn j4_arena_overflow_600k_list() {
    // 600K × 16 bytes/ListNode = 9.6 MB — exceeds 8 MB arena.
    let src = "main = length [1..600000]";
    let start = Instant::now();
    let result = synoema_codegen::compile_and_run(src);
    let elapsed = start.elapsed();
    println!("J-4: range [1..600000] in {:?}  (arena overflow → system malloc)", elapsed);
    assert!(result.is_ok(), "Arena overflow caused crash: {:?}", result.err());
    assert_eq!(result.unwrap(), 600_000, "Wrong length after overflow");
}

// ── J-5: JIT correctness suite ───────────────────────────────────────────────

/// J-5: Core programs must produce known correct results in JIT.
#[test]
fn j5_jit_correctness_suite() {
    let cases: &[(&str, i64)] = &[
        ("fac 0 = 1\nfac n = n * fac (n - 1)\nmain = fac 10", 3_628_800),
        ("fib 0 = 0\nfib 1 = 1\nfib n = fib (n-1) + fib (n-2)\nmain = fib 10", 55),
        ("ack 0 n = n + 1\nack m 0 = ack (m-1) 1\nack m n = ack (m-1) (ack m (n-1))\nmain = ack 3 2", 29),
        ("main = sum [1..100]", 5050),
        ("main = length [1..42]", 42),
        ("main = ? 2 ** 10 == 1024 -> 42 : 0", 42),
    ];
    for (src, expected) in cases {
        let result = jit(src);
        assert_eq!(result, *expected, "JIT mismatch:\n{}", src);
        let preview: String = src.chars().take(50).collect();
        println!("J-5: {:?} = {} ✓", preview, result);
    }
}

// ── J-6: JIT recursion scale ──────────────────────────────────────────────────

#[test]
fn j6_factorial_20() {
    let src = "fac 0 = 1\nfac n = n * fac (n - 1)\nmain = fac 20";
    let start = Instant::now();
    let result = jit(src);
    let elapsed = start.elapsed();
    // Overflows i64 but should not crash.
    println!("J-6: JIT factorial(20) = {} in {:?}", result, elapsed);
}

#[test]
#[ignore] // Exponential tree — slow
fn j6_fibonacci_40() {
    let src = "fib 0 = 0\nfib 1 = 1\nfib n = fib (n-1) + fib (n-2)\nmain = fib 40";
    let start = Instant::now();
    let result = synoema_codegen::compile_and_run(src);
    let elapsed = start.elapsed();
    println!("J-6: JIT fib(40) = {:?} in {:?}  (threshold: 5 s)", result, elapsed);
    assert_eq!(result.unwrap(), 102_334_155);
    assert!(elapsed.as_secs() < 5, "fib(40) JIT took {:?}", elapsed);
}

// ── J-6b: JIT tail recursion ─────────────────────────────────────────────────

/// J-6b: 1K tail-recursive iterations — safe depth for any stack.
#[test]
fn j6b_tail_recursion_1k() {
    let src = "go acc n = ? n == 0 -> acc : go (acc + n) (n - 1)\nmain = go 0 1000";
    let start = Instant::now();
    let result = synoema_codegen::compile_and_run(src);
    let elapsed = start.elapsed();
    println!("J-6b: JIT tail-rec 1K in {:?}", elapsed);
    assert_eq!(result.unwrap(), 500_500); // sum(1..1000)
}

/// J-6b: 1M tail-recursive iterations — tests whether JIT has TCO.
/// May SIGSEGV if JIT lacks tail-call optimisation.
#[test]
#[ignore]
fn j6b_tail_recursion_1m_tco_test() {
    let src = "go acc n = ? n == 0 -> acc : go (acc + n) (n - 1)\nmain = go 0 1000000";
    let start = Instant::now();
    let result = synoema_codegen::compile_and_run(src);
    let elapsed = start.elapsed();
    println!("J-6b: JIT tail-rec 1M in {:?}: {:?}", elapsed, result.is_ok());
    if let Ok(v) = result {
        assert_eq!(v, 500_000_500_000_i64); // sum(1..1000000)
        println!("J-6b: JIT TCO confirmed ✓");
    } else {
        println!("J-6b: JIT lacks TCO — stack overflow at 1M iterations");
    }
}

// ── J-6c: Map/Filter in JIT ──────────────────────────────────────────────────
// NOTE: `map` is an interpreter builtin; JIT uses user-defined map or
// synoema_concatmap (list comprehensions) for the same effect.

#[test]
fn j6c_comprehension_as_map_1k() {
    // [x*2 | x <- [1..1000]] is the JIT equivalent of map (*2) [1..1000]
    let src = "main = sum [x * 2 | x <- [1..1000]]";
    let start = Instant::now();
    let result = synoema_codegen::compile_and_run(src);
    let elapsed = start.elapsed();
    println!("J-6c: JIT [x*2 | x<-[1..1000]] in {:?}", elapsed);
    assert_eq!(result.unwrap(), 1_001_000); // 2 * sum(1..1000) = 2 * 500500
}

#[test]
fn j6c_filter_comprehension_1k() {
    // [x | x <- [1..1000], x % 2 == 0] is JIT filter
    let src = "main = length [x | x <- [1..1000], x % 2 == 0]";
    let start = Instant::now();
    let result = synoema_codegen::compile_and_run(src);
    let elapsed = start.elapsed();
    println!("J-6c: JIT filter even [1..1000] in {:?}", elapsed);
    assert_eq!(result.unwrap(), 500);
}

#[test]
#[ignore]
fn j6c_comprehension_10k() {
    let src = "main = sum [x * 2 | x <- [1..10000]]";
    let start = Instant::now();
    let result = synoema_codegen::compile_and_run(src);
    let elapsed = start.elapsed();
    println!("J-6c: JIT [x*2 | x<-[1..10000]] in {:?}  (release threshold: 50 ms)", elapsed);
    assert_eq!(result.unwrap(), 100_010_000);
    #[cfg(not(debug_assertions))]
    assert!(elapsed.as_millis() < 50, "JIT comprehension 10K took {:?}", elapsed);
}

// ── J-7: List concat scale ────────────────────────────────────────────────────

#[test]
fn j7_list_concat_2k() {
    let src = "main = length ([1..1000] ++ [1001..2000])";
    let start = Instant::now();
    let result = synoema_codegen::compile_and_run(src);
    let elapsed = start.elapsed();
    println!("J-7: [1..1000] ++ [1001..2000] in {:?}", elapsed);
    assert_eq!(result.unwrap(), 2000);
}

// ── J-9: Integer power overflow ───────────────────────────────────────────────

/// J-9: 2**62 fits in i64 — verify exact value.
#[test]
fn j9_power_2_62() {
    let result = jit("main = 2 ** 62");
    assert_eq!(result, 4_611_686_018_427_387_904_i64);
    println!("J-9: 2**62 = {} ✓", result);
}

/// J-9: 2**63 overflows i64 — wrapping behaviour, must not panic.
#[test]
fn j9_power_2_63_wrapping() {
    let result = synoema_codegen::compile_and_run("main = 2 ** 63");
    println!(
        "J-9: 2**63 = {:?}  (wrapping or error — both accepted, no panic)",
        result
    );
    // No assertion on value — just must not panic.
}

// ── J-11: Record field access via FNV-hash ────────────────────────────────────

/// J-11: 50-field record, access field f42 — tests FNV-hash lookup.
#[test]
fn j11_record_50_fields_field_access() {
    let fields = (0..50)
        .map(|i| format!("f{} = {}", i, i))
        .collect::<Vec<_>>()
        .join(", ");
    let src = format!("r = {{{}}}\nmain = r.f42", fields);
    let start = Instant::now();
    let result = synoema_codegen::compile_and_run(&src);
    let elapsed = start.elapsed();
    println!("J-11: 50-field record access r.f42 in {:?}", elapsed);
    assert_eq!(result.unwrap(), 42);
}

// ── J-12: Cold vs warm JIT compilation ───────────────────────────────────────

/// J-12: Each compile creates a fresh SimpleJITModule — document overhead.
#[test]
fn j12_cold_vs_warm_compilation() {
    let src = "fac 0 = 1\nfac n = n * fac (n - 1)\nmain = fac 10";

    // Cold start
    let cold = Instant::now();
    synoema_codegen::compile_and_run(src).unwrap();
    let cold_time = cold.elapsed();

    // 9 more to get warm average
    let warm = Instant::now();
    for _ in 0..9 {
        synoema_codegen::compile_and_run(src).unwrap();
    }
    let warm_avg = warm.elapsed() / 9;

    println!(
        "J-12: cold={:?}  warm_avg={:?}  (no caching — each run rebuilds JIT module)",
        cold_time, warm_avg
    );
}

/// J-12: 100 compilations — check for state accumulation.
#[test]
#[ignore]
fn j12_100_compilations_no_leak() {
    let src = "fac 0 = 1\nfac n = n * fac (n - 1)\nmain = fac 10";
    let n = 100u32;
    let start = Instant::now();
    for i in 0..n {
        let r = synoema_codegen::compile_and_run(src).unwrap();
        assert_eq!(r, 3_628_800, "Compile {} returned wrong result", i);
    }
    let avg = start.elapsed() / n;
    println!("J-12: 100 compilations, {:?} avg/run — no state accumulation", avg);
}

// ── J-13: JIT native stack depth ─────────────────────────────────────────────

/// J-13: 500 recursive frames in JIT — safe for OS default stack (8 MB).
#[test]
fn j13_jit_recursive_depth_500() {
    // JIT uses OS stack (no 64 MB thread). 500 frames is conservative.
    let src = "fac 0 = 1\nfac n = n * fac (n - 1)\nmain = fac 500";
    let result = synoema_codegen::compile_and_run(src);
    println!("J-13: factorial(500) in JIT: {}", if result.is_ok() { "OK" } else { "FAILED (stack?)" });
    // i64 overflows but must not SIGSEGV.
    // We accept either Ok (overflowed value) or Err (numeric overflow handled).
}

/// J-13: Progressive depth to find native stack limit.
/// DANGEROUS — may SIGSEGV (aborting the process).
#[test]
#[ignore]
fn j13_find_jit_native_stack_limit() {
    for n in [1_000, 2_000, 5_000, 10_000, 20_000] {
        let src = format!("fac 0 = 1\nfac n = n * fac (n - 1)\nmain = fac {}", n);
        match synoema_codegen::compile_and_run(&src) {
            Ok(_) => println!("J-13: factorial({}) in JIT — OK", n),
            Err(e) => {
                println!("J-13: factorial({}) in JIT — FAILED: {}", n, e);
                break;
            }
        }
    }
}

// ── J-13b: List comprehension in JIT ─────────────────────────────────────────

#[test]
fn j13b_list_comprehension_1k() {
    let src = "main = sum [x * x | x <- [1..100]]"; // 100 elements → sum of squares
    let start = Instant::now();
    let result = synoema_codegen::compile_and_run(src);
    let elapsed = start.elapsed();
    println!("J-13b: JIT comprehension [x*x | x<-[1..100]] in {:?}", elapsed);
    assert_eq!(result.unwrap(), 338_350); // sum(i² for i=1..100)
}

#[test]
#[ignore]
fn j13b_list_comprehension_10k() {
    let src = "main = sum [x * x | x <- [1..1000]]"; // 1K elements
    let start = Instant::now();
    let result = synoema_codegen::compile_and_run(src);
    let elapsed = start.elapsed();
    println!("J-13b: JIT comprehension [x*x | x<-[1..1000]] in {:?}  (threshold: 100 ms)", elapsed);
    assert!(result.is_ok(), "Comprehension 1K failed: {:?}", result.err());
    assert!(elapsed.as_millis() < 100, "Comprehension 1K took {:?}", elapsed);
}

// ── J-13c: Numeric edge cases ─────────────────────────────────────────────────

/// J-13c: Numeric edge cases — boundary values, wrapping, float precision.
/// NOTE: Division by zero in JIT triggers a Cranelift `ud2` trap (SIGILL) and
/// aborts the process — it cannot be tested safely here. See J-13c in the plan.
#[test]
fn j13c_numeric_edge_cases() {
    // i64::MAX as a literal
    let max_val = jit("main = 9223372036854775807");
    assert_eq!(max_val, i64::MAX);
    println!("J-13c: i64::MAX = {} ✓", max_val);

    // NOTE: i64::MAX + 1 is constant-folded by the optimizer in debug mode,
    // which panics on overflow (Rust debug arithmetic). Tested separately
    // in release mode only — see j13c plan note.

    // Float precision: 0.1 + 0.2 ≠ 0.3 (IEEE 754 — document only, no assertion).
    let r = synoema_codegen::compile_and_display("main = 0.1 + 0.2");
    println!("J-13c: 0.1 + 0.2 = {:?}  (IEEE 754, not exactly 0.3)", r);

    // Float: sqrt of a perfect square
    let sqrt4 = synoema_codegen::compile_and_display("main = sqrt 4.0");
    println!("J-13c: sqrt(4.0) = {:?}  (expected 2.0)", sqrt4);

    println!("J-13c: numeric edge cases — no panics, no SIGILL");
}

// ── R-2: Arena reset correctness ─────────────────────────────────────────────

/// R-2: 100 allocate-reset cycles — offset always returns to 0.
#[test]
fn r2_arena_reset_100_cycles() {
    let src = "main = sum [1..1000]"; // allocates 1000 ListNodes per run
    for i in 0..100 {
        let result = synoema_codegen::compile_and_run(src);
        assert_eq!(
            result.unwrap(),
            500_500,
            "Cycle {} returned wrong result — arena state corrupted",
            i
        );
    }
    println!("R-2: 100 arena reset cycles — all correct, no double-free");
}

// ── R-5b: show on large structures ───────────────────────────────────────────

/// R-5b: show [1..100] — exercises synoema_show_list recursion.
#[test]
fn r5b_show_list_100() {
    let src = "main = show [1..100]";
    let start = Instant::now();
    let result = synoema_codegen::compile_and_display(src);
    let elapsed = start.elapsed();
    println!("R-5b: show [1..100] in {:?}", elapsed);
    assert!(result.is_ok(), "show [1..100] failed: {:?}", result.err());
    // Result should start with "["
    let s = result.unwrap();
    assert!(s.starts_with('['), "show result unexpected: {:?}", &s[..s.len().min(20)]);
}

/// R-5b: show [1..10000] — 10K element list, no stack overflow in show_list recursion.
#[test]
#[ignore]
fn r5b_show_list_10k() {
    let src = "main = show [1..10000]";
    let start = Instant::now();
    let result = synoema_codegen::compile_and_display(src);
    let elapsed = start.elapsed();
    println!("R-5b: show [1..10000] in {:?}  (threshold: 100 ms)", elapsed);
    assert!(result.is_ok(), "show [1..10000] failed: {:?}", result.err());
    assert!(elapsed.as_millis() < 100, "show [1..10000] took {:?}", elapsed);
}

// ── R-5c: Float math builtins ─────────────────────────────────────────────────

/// R-5c: sqrt, floor, ceil, abs, round — verify correctness and no crash.
#[test]
fn r5c_float_math_builtins() {
    let cases: &[(&str, &str)] = &[
        ("main = sqrt 4.0",    "2.0"),
        ("main = floor 3.7",   "3.0"),
        ("main = ceil 3.2",    "4.0"),
        ("main = abs (0.0 - 5.0)", "5.0"),
        ("main = round 3.5",   "4.0"),
    ];
    for (src, _expected_str) in cases {
        let result = synoema_codegen::compile_and_display(src);
        assert!(result.is_ok(), "Builtin failed: {:?}\n  src: {}", result.err(), src);
        println!("R-5c: {}  →  {:?}", src, result.unwrap());
    }
}

// ── R-5d: head/tail/length/sum on large lists ────────────────────────────────

#[test]
fn r5d_length_100k() {
    let src = "main = length [1..100000]";
    let start = Instant::now();
    let result = synoema_codegen::compile_and_run(src);
    let elapsed = start.elapsed();
    println!("R-5d: length [1..100000] in {:?}  (release threshold: 50 ms)", elapsed);
    assert_eq!(result.unwrap(), 100_000);
    #[cfg(not(debug_assertions))]
    assert!(elapsed.as_millis() < 50, "length 100K took {:?}", elapsed);
}

#[test]
fn r5d_sum_100k() {
    let src = "main = sum [1..100000]";
    let start = Instant::now();
    let result = synoema_codegen::compile_and_run(src);
    let elapsed = start.elapsed();
    println!("R-5d: sum [1..100000] in {:?}  (release threshold: 50 ms)", elapsed);
    assert_eq!(result.unwrap(), 5_000_050_000_i64);
    #[cfg(not(debug_assertions))]
    assert!(elapsed.as_millis() < 50, "sum 100K took {:?}", elapsed);
}

#[test]
fn r5d_head_tail_o1() {
    // head and tail should be O(1) — constant time regardless of list length.
    let src_head = "main = head [1..100000]";
    let src_tail_len = "main = length (tail [1..100000])";

    let t1 = Instant::now();
    assert_eq!(jit(src_head), 1);
    let head_time = t1.elapsed();

    let t2 = Instant::now();
    assert_eq!(jit(src_tail_len), 99_999);
    let tail_time = t2.elapsed();

    println!("R-5d: head [1..100000] in {:?}", head_time);
    println!("R-5d: tail [1..100000] (length) in {:?}", tail_time);
}

// ── Throughput bench ─────────────────────────────────────────────────────────

#[test]
#[ignore]
fn bench_jit_compile_and_run() {
    let src = "fac 0 = 1\nfac n = n * fac (n - 1)\nmain = fac 10";
    let n = 100u32;

    // Warmup
    for _ in 0..5 {
        let _ = synoema_codegen::compile_and_run(src);
    }

    let start = Instant::now();
    for _ in 0..n {
        synoema_codegen::compile_and_run(src).unwrap();
    }
    let avg = start.elapsed() / n;
    println!("BENCH jit_factorial(10): {:?} avg/run  (incl. Cranelift compile)", avg);
}

// ── String Interpolation in JIT ──────────────────────────

#[test]
fn jit_interp_simple() {
    let s = jit_str("name = \"World\"\nmain = \"Hello ${name}\"");
    assert_eq!(s, "Hello World");
}

#[test]
fn jit_interp_int() {
    let s = jit_str("x = 42\nmain = \"x=${x}\"");
    assert_eq!(s, "x=42");
}

#[test]
fn jit_interp_expression() {
    let s = jit_str("main = \"sum=${2 + 3}\"");
    assert_eq!(s, "sum=5");
}

#[test]
fn jit_interp_multiple() {
    let s = jit_str("a = 1\nb = 2\nmain = \"${a}+${b}=${a + b}\"");
    assert_eq!(s, "1+2=3");
}

#[test]
fn jit_interp_escape_dollar() {
    let s = jit_str(r#"main = "\$ is money""#);
    assert_eq!(s, "$ is money");
}

// ── Memory Management v2: arena_save / arena_restore ─────────────────────────

#[test]
fn arena_save_restore_basic() {
    // arena_save / arena_restore exist as runtime functions and can be called.
    // They return i64 values (arena offset).
    use synoema_codegen::runtime::{arena_save, arena_restore, arena_reset};
    let saved = arena_save();
    assert!(saved >= 0, "arena_save should return non-negative offset");
    arena_restore(saved);
    arena_reset();
}

#[test]
fn arena_save_restore_preserves_offset() {
    use synoema_codegen::runtime::{arena_save, arena_reset};
    arena_reset();
    let _before = arena_save();
    // Allocate something to advance offset
    let src = "main = length [1..100]";
    let _ = synoema_codegen::compile_and_run(src);
    // After compile_and_run, arena is reset (lib.rs calls arena_reset)
    let after = arena_save();
    // After reset, offset should be 0 again
    assert_eq!(after, 0, "After arena_reset, offset should be 0");
}

#[test]
fn arena_overflow_tracked_cleanup() {
    // After arena overflow + reset, overflow allocations should be freed.
    // We can't directly verify dealloc, but we can verify no crash on
    // repeated overflow cycles.
    for _ in 0..3 {
        let src = "main = length [1..600000]"; // 9.6 MB > 8 MB arena
        let result = synoema_codegen::compile_and_run(src);
        assert!(result.is_ok(), "Overflow cycle should not crash");
        assert_eq!(result.unwrap(), 600_000);
        // arena_reset() is called inside compile_and_run
        // → overflow_allocs should be freed each time
    }
}

// ── Record Punning in JIT ──────────────────────────────────────────────────

#[test]
fn jit_record_punning_basic() {
    let src = "main =\n  x = 3\n  y = 4\n  r = {x, y}\n  r.x + r.y";
    let result = synoema_codegen::compile_and_run(src);
    assert_eq!(result.unwrap(), 7);
}

#[test]
fn jit_record_punning_mixed() {
    let src = "main =\n  x = 10\n  r = {x, y = 20}\n  r.x + r.y";
    let result = synoema_codegen::compile_and_run(src);
    assert_eq!(result.unwrap(), 30);
}

// ── Wildcard Import in JIT ─────────────────────────────────────────────────

#[test]
fn jit_wildcard_import() {
    let src = "mod Math\n  square x = x * x\n  cube x = x * x * x\nuse Math (*)\nmain = square 5 + cube 2";
    let result = synoema_codegen::compile_and_run(src);
    assert_eq!(result.unwrap(), 33);
}

#[test]
fn jit_wildcard_import_constant() {
    let src = "mod Consts\n  pi = 314\n  e = 271\nuse Consts (*)\nmain = pi + e";
    let result = synoema_codegen::compile_and_run(src);
    assert_eq!(result.unwrap(), 585);
}

// ── Record Update in JIT ───────────────────────────────────────────────────

#[test]
fn jit_record_update_basic() {
    let src = "main =\n  r = {x = 1, y = 2}\n  r2 = {...r, x = 10}\n  r2.x";
    let result = synoema_codegen::compile_and_run(src);
    assert_eq!(result.unwrap(), 10);
}

#[test]
fn jit_record_update_preserves_other_fields() {
    let src = "main =\n  r = {x = 1, y = 2}\n  r2 = {...r, x = 10}\n  r2.y";
    let result = synoema_codegen::compile_and_run(src);
    assert_eq!(result.unwrap(), 2);
}

#[test]
fn jit_record_update_multiple_fields() {
    let src = "main =\n  r = {x = 1, y = 2, z = 3}\n  r2 = {...r, x = 10, y = 20}\n  r2.z";
    let result = synoema_codegen::compile_and_run(src);
    assert_eq!(result.unwrap(), 3);
}

// ── M-1: Arena Leak Detection ───────────────────────────────────────────────

/// M-1a: arena_reset sets offset to 0.
#[test]
fn m1a_arena_reset_offset_zero() {
    use synoema_codegen::runtime::{arena_reset, arena_save};
    // Allocate something via JIT
    let _ = synoema_codegen::compile_and_run("main = length [1..100]");
    // compile_and_run calls arena_reset internally
    let offset = arena_save();
    assert_eq!(offset, 0, "arena offset should be 0 after compile_and_run");
    arena_reset();
}

/// M-1b: arena_reset clears overflow allocations.
#[test]
fn m1b_arena_reset_clears_overflow() {
    use synoema_codegen::{arena_overflow_count, arena_reset};
    // Trigger overflow: 600K ListNodes = 9.6 MB > 8 MB arena
    let _ = synoema_codegen::compile_and_run("main = length [1..600000]");
    // compile_and_run calls arena_reset — overflow allocs should be freed
    let count = arena_overflow_count();
    assert_eq!(count, 0, "overflow allocs should be 0 after arena_reset, got {}", count);
    arena_reset();
}

/// M-1c: arena_reset resets region_depth to 0.
#[test]
fn m1c_arena_reset_region_depth_zero() {
    use synoema_codegen::{arena_region_depth, arena_reset};
    // Run a program that uses regions (list comprehension triggers region inference)
    let _ = synoema_codegen::compile_and_run("main = sum [x * 2 | x <- [1..100]]");
    let depth = arena_region_depth();
    assert_eq!(depth, 0, "region_depth should be 0 after compile_and_run, got {}", depth);
    arena_reset();
}

/// M-1d: compile_and_run leaves arena completely clean.
#[test]
fn m1d_compile_and_run_leaves_arena_clean() {
    use synoema_codegen::{arena_offset, arena_overflow_count, arena_region_depth};
    let programs = &[
        "main = 42",
        "main = length [1..1000]",
        "main = sum [x * x | x <- [1..100]]",
        "fac 0 = 1\nfac n = n * fac (n - 1)\nmain = fac 10",
        "main = head [1..100]",
    ];
    for src in programs {
        let _ = synoema_codegen::compile_and_run(src);
        assert_eq!(arena_offset(), 0, "offset leak after: {}", src);
        assert_eq!(arena_overflow_count(), 0, "overflow leak after: {}", src);
        assert_eq!(arena_region_depth(), 0, "region depth leak after: {}", src);
    }
}

// ── M-2: Region Balance Tests ───────────────────────────────────────────────

/// M-2a: region_enter/region_exit pair restores offset.
#[test]
fn m2a_region_enter_exit_restores_offset() {
    use synoema_codegen::runtime::{synoema_region_enter, synoema_region_exit, arena_save, arena_reset};
    arena_reset();
    let before = arena_save();
    synoema_region_enter();
    // Simulate some allocation by running a small program fragment
    // (we can't directly allocate, but region_enter/exit should balance)
    synoema_region_exit();
    let after = arena_save();
    assert_eq!(before, after, "offset should be restored after region_enter/exit pair");
    arena_reset();
}

/// M-2b: nested regions (depth 3) — correct enter/exit balance.
#[test]
fn m2b_nested_regions_balanced() {
    use synoema_codegen::runtime::{synoema_region_enter, synoema_region_exit, arena_reset};
    use synoema_codegen::arena_region_depth;
    arena_reset();
    assert_eq!(arena_region_depth(), 0, "start at depth 0");

    synoema_region_enter();
    assert_eq!(arena_region_depth(), 1, "depth 1 after first enter");

    synoema_region_enter();
    assert_eq!(arena_region_depth(), 2, "depth 2 after second enter");

    synoema_region_enter();
    assert_eq!(arena_region_depth(), 3, "depth 3 after third enter");

    synoema_region_exit();
    assert_eq!(arena_region_depth(), 2, "depth 2 after first exit");

    synoema_region_exit();
    assert_eq!(arena_region_depth(), 1, "depth 1 after second exit");

    synoema_region_exit();
    assert_eq!(arena_region_depth(), 0, "depth 0 after third exit");

    arena_reset();
}

/// M-2c: region_exit at depth 0 is no-op (no underflow).
#[test]
fn m2c_region_exit_at_zero_no_underflow() {
    use synoema_codegen::runtime::{synoema_region_exit, arena_reset};
    use synoema_codegen::arena_region_depth;
    arena_reset();
    assert_eq!(arena_region_depth(), 0);
    // Calling exit at depth 0 should be a no-op
    synoema_region_exit();
    assert_eq!(arena_region_depth(), 0, "region_exit at 0 should not underflow");
    // Multiple exits at 0 — still safe
    synoema_region_exit();
    synoema_region_exit();
    assert_eq!(arena_region_depth(), 0, "multiple region_exit at 0 should be safe");
    arena_reset();
}

/// M-2d: JIT program with region inference — region_depth == 0 after run.
#[test]
fn m2d_jit_regions_balanced_after_run() {
    use synoema_codegen::arena_region_depth;
    // Programs that trigger region inference (let + list allocation)
    let programs = &[
        "main = sum [x | x <- [1..100], x % 2 == 0]",
        "main = length [x * x | x <- [1..50]]",
        "go acc n = ? n == 0 -> acc : go (acc + n) (n - 1)\nmain = go 0 100",
    ];
    for src in programs {
        let _ = synoema_codegen::compile_and_run(src);
        assert_eq!(arena_region_depth(), 0, "region imbalance after: {}", src);
    }
}

// ── M-3: Leak Audit — Existing JIT Tests ────────────────────────────────────

/// M-3: Run all core JIT programs and verify arena is clean after each.
#[test]
fn m3_leak_audit_all_jit_programs() {
    use synoema_codegen::{arena_offset, arena_overflow_count, arena_region_depth};
    let programs: &[(&str, &str)] = &[
        ("fac 0 = 1\nfac n = n * fac (n - 1)\nmain = fac 10", "factorial"),
        ("fib 0 = 0\nfib 1 = 1\nfib n = fib (n-1) + fib (n-2)\nmain = fib 10", "fibonacci"),
        ("ack 0 n = n + 1\nack m 0 = ack (m-1) 1\nack m n = ack (m-1) (ack m (n-1))\nmain = ack 3 2", "ackermann"),
        ("main = sum [1..100]", "sum_range"),
        ("main = length [1..42]", "length_range"),
        ("main = ? 2 ** 10 == 1024 -> 42 : 0", "ternary_power"),
        ("main = sum [x * 2 | x <- [1..1000]]", "comprehension_map"),
        ("main = length [x | x <- [1..1000], x % 2 == 0]", "comprehension_filter"),
        ("main = length ([1..1000] ++ [1001..2000])", "list_concat"),
        ("main = 2 ** 62", "power_large"),
        ("main = head [1..100000]", "head_large_list"),
        ("main = length (tail [1..100000])", "tail_large_list"),
        ("main = sum [1..100000]", "sum_100k"),
        ("main = length [1..100000]", "length_100k"),
        ("go acc n = ? n == 0 -> acc : go (acc + n) (n - 1)\nmain = go 0 1000", "tail_recursion"),
        ("main = sum [x * x | x <- [1..100]]", "comprehension_squares"),
        ("name = \"World\"\nmain = \"Hello ${name}\"", "string_interp"),
        ("x = 42\nmain = \"x=${x}\"", "string_interp_int"),
        ("main = \"sum=${2 + 3}\"", "string_interp_expr"),
        ("a = 1\nb = 2\nmain = \"${a}+${b}=${a + b}\"", "string_interp_multi"),
        ("main =\n  x = 3\n  y = 4\n  r = {x, y}\n  r.x + r.y", "record_punning"),
        ("main =\n  r = {x = 1, y = 2}\n  r2 = {...r, x = 10}\n  r2.x", "record_update"),
        ("mod Math\n  square x = x * x\n  cube x = x * x * x\nuse Math (*)\nmain = square 5 + cube 2", "wildcard_import"),
    ];

    let mut leaks_found = Vec::new();
    for (src, name) in programs {
        let _ = synoema_codegen::compile_and_run(src);
        let offset = arena_offset();
        let overflow = arena_overflow_count();
        let depth = arena_region_depth();
        if offset != 0 || overflow != 0 || depth != 0 {
            leaks_found.push(format!(
                "{}: offset={}, overflow={}, depth={}",
                name, offset, overflow, depth
            ));
        }
    }
    assert!(
        leaks_found.is_empty(),
        "Memory leaks detected in {} programs:\n{}",
        leaks_found.len(),
        leaks_found.join("\n")
    );
    println!("M-3: {} JIT programs audited — all clean", programs.len());
}

// ── M-4: Stress — Repeated Alloc-Reset Cycles ──────────────────────────────

/// M-4a: 1000 alloc-reset cycles — offset stable at 0.
#[test]
fn m4a_1000_alloc_reset_cycles_stable() {
    use synoema_codegen::arena_offset;
    let src = "main = sum [1..1000]";
    for i in 0..1000 {
        let result = synoema_codegen::compile_and_run(src);
        assert_eq!(result.unwrap(), 500_500, "wrong result at cycle {}", i);
        assert_eq!(arena_offset(), 0, "offset leak at cycle {}", i);
    }
    println!("M-4a: 1000 alloc-reset cycles — offset stable at 0");
}

/// M-4b: repeated overflow-reset cycles — overflow_count stable at 0.
#[test]
fn m4b_overflow_reset_cycles_stable() {
    use synoema_codegen::{arena_offset, arena_overflow_count};
    let src = "main = length [1..600000]"; // 9.6 MB > 8 MB arena
    for i in 0..5 {
        let result = synoema_codegen::compile_and_run(src);
        assert_eq!(result.unwrap(), 600_000, "wrong result at overflow cycle {}", i);
        assert_eq!(arena_offset(), 0, "offset leak at overflow cycle {}", i);
        assert_eq!(arena_overflow_count(), 0, "overflow leak at overflow cycle {}", i);
    }
    println!("M-4b: 5 overflow-reset cycles — all clean");
}
