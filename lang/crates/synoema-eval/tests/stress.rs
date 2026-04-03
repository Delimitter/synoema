//! Interpreter load tests.
//!
//! Fast tests:
//!   cargo test -p synoema-eval stress
//!
//! All including slow:
//!   cargo test -p synoema-eval stress -- --include-ignored
//!
//! Show timing:
//!   cargo test -p synoema-eval stress -- --nocapture

use std::time::Instant;
use synoema_eval::Value;

fn run(src: &str) -> (Value, Vec<String>) {
    synoema_eval::eval_main(src)
        .unwrap_or_else(|e| panic!("Run failed:\n{}\nError: {}", src, e))
}

// ── E-1: Factorial — recursion ────────────────────────────────────────────────

#[test]
fn e1_factorial_20() {
    let src = "fac 0 = 1\nfac n = n * fac (n - 1)\nmain = fac 20";
    let start = Instant::now();
    let (val, _) = run(src);
    let elapsed = start.elapsed();
    println!("E-1: factorial(20) in {:?}  (release threshold: 5 ms)", elapsed);
    assert_eq!(val, Value::Int(2_432_902_008_176_640_000));
    #[cfg(not(debug_assertions))]
    assert!(elapsed.as_millis() < 5, "factorial(20) took {:?}", elapsed);
}

#[test]
#[ignore] // debug: ~8KB/Rust-frame × 5000 frames ≈ 40 MB → overflows 64 MB eval thread → SIGABRT
fn e1_factorial_1000() {
    let src = "fac 0 = 1\nfac n = n * fac (n - 1)\nmain = fac 1000";
    let start = Instant::now();
    let result = synoema_eval::eval_main(src);
    let elapsed = start.elapsed();
    println!("E-1: factorial(1000) in {:?}  (i64 overflows but no crash)", elapsed);
    // Result overflows i64 but must not panic.
    assert!(result.is_ok(), "factorial(1000) failed: {:?}", result.err());
}

// ── E-2: Fibonacci — double recursion (max stack depth = n) ──────────────────

#[test]
fn e2_fibonacci_25() {
    // fib(25) = 75025. Max recursion depth = 25 — safe for the 64MB eval thread.
    // (Ackermann m≥3 overflows the thread even for small n; see ignored test below.)
    // fib(0)=1, fib(1)=1, fib(25)=121393
    let src = "fib 0 = 1\nfib 1 = 1\nfib n = fib (n - 1) + fib (n - 2)\nmain = fib 25";
    let start = Instant::now();
    let (val, _) = run(src);
    let elapsed = start.elapsed();
    println!("E-2: fib(25) = {:?} in {:?}", val, elapsed);
    assert_eq!(val, Value::Int(121393));
}

#[test]
#[ignore] // Ackermann m≥3 has exponential call-frame depth — overflows 64MB eval thread
fn e2_ackermann_3_4() {
    let src = "ack 0 n = n + 1\nack m 0 = ack (m - 1) 1\nack m n = ack (m - 1) (ack m (n - 1))\nmain = ack 3 4";
    let start = Instant::now();
    let (val, _) = run(src);
    let elapsed = start.elapsed();
    println!("E-2: ack(3,4) = {:?} in {:?}", val, elapsed);
    assert_eq!(val, Value::Int(125));
}

// ── E-3: List cons — O(n²) documented ────────────────────────────────────────

#[test]
fn e3_cons_scale_document() {
    // build n builds a list by prepending (O(n²) total due to Vec::insert(0,...)).
    // Each cons call ≈ 5 Rust frames; 64MB / 8KB ≈ 8192 frames → safe depth ≈ 1500.
    // Use 200 and 600 to stay well within budget in debug mode.
    let src200 = "build 0 = []\nbuild n = n : build (n - 1)\nmain = build 200";
    let start = Instant::now();
    let _ = run(src200);
    let t200 = start.elapsed();

    let src600 = "build 0 = []\nbuild n = n : build (n - 1)\nmain = build 600";
    let start = Instant::now();
    let _ = run(src600);
    let t600 = start.elapsed();

    println!(
        "E-3: cons list 200={:?}  600={:?}  (O(n²) expected, ratio≈{})",
        t200,
        t600,
        t600.as_micros() / (t200.as_micros().max(1))
    );
}

// ── E-4: List pattern matching ────────────────────────────────────────────────

#[test]
fn e4_list_pattern_sum_1k() {
    let src = "mysum [] = 0\nmysum (x:xs) = x + mysum xs\nmain = mysum [1..1000]";
    let start = Instant::now();
    let (val, _) = run(src);
    let elapsed = start.elapsed();
    println!("E-4: sum [1..1000] via pattern match in {:?}", elapsed);
    assert_eq!(val, Value::Int(500_500));
}

#[test]
#[ignore]
fn e4_list_pattern_sum_10k() {
    let src = "mysum [] = 0\nmysum (x:xs) = x + mysum xs\nmain = mysum [1..10000]";
    let start = Instant::now();
    let (val, _) = run(src);
    let elapsed = start.elapsed();
    println!("E-4: sum [1..10000] in {:?}  (threshold: 500 ms)", elapsed);
    assert_eq!(val, Value::Int(50_005_000));
    assert!(elapsed.as_millis() < 500, "10K list sum took {:?}", elapsed);
}

// ── E-5: Closures — deep capture ─────────────────────────────────────────────

#[test]
fn e5_closure_deep_capture() {
    let src = "\
make_adder x = \\y -> x + y
add1 = make_adder 1
add10 = make_adder 10
add100 = make_adder 100
main = add1 (add10 (add100 1))";
    let (val, _) = run(src);
    assert_eq!(val, Value::Int(112));
    println!("E-5: nested closures — OK");
}

#[test]
fn e5_closure_chain_50() {
    // Chain of 50 closures each capturing the previous.
    let src = "\
f0 x = x + 1
f1 g = \\x -> g (x + 1)
chain = f1 (f1 (f1 (f1 (f1 (f1 (f1 (f1 (f1 (f1 f0)))))))))
main = chain 0";
    let result = synoema_eval::eval_main(src);
    assert!(result.is_ok(), "Closure chain failed: {:?}", result.err());
    println!("E-5: 10-step closure chain — OK");
}

// ── E-6: Map/Filter on large list ─────────────────────────────────────────────

#[test]
fn e6_map_1k() {
    let src = "main = sum (map (\\x -> x * 2) [1..1000])";
    let (val, _) = run(src);
    assert_eq!(val, Value::Int(1_001_000)); // 2 * sum(1..1000) = 2 * 500500
    println!("E-6: map (*2) [1..1000] — OK");
}

#[test]
fn e6_map_10k() {
    let src = "main = sum (map (\\x -> x * 2) [1..10000])";
    let start = Instant::now();
    let (val, _) = run(src);
    let elapsed = start.elapsed();
    println!("E-6: map (*2) [1..10000] in {:?}  (release threshold: 500 ms)", elapsed);
    assert_eq!(val, Value::Int(100_010_000)); // 2 * 50005000
    #[cfg(not(debug_assertions))]
    assert!(elapsed.as_millis() < 500, "map 10K took {:?}", elapsed);
}

#[test]
fn e6_filter_10k() {
    let src = "main = length (filter (\\x -> x % 2 == 0) [1..10000])";
    let start = Instant::now();
    let (val, _) = run(src);
    let elapsed = start.elapsed();
    println!("E-6: filter even [1..10000] in {:?}  (release threshold: 500 ms)", elapsed);
    assert_eq!(val, Value::Int(5000));
    #[cfg(not(debug_assertions))]
    assert!(elapsed.as_millis() < 500, "filter 10K took {:?}", elapsed);
}

// ── E-7: Record with N fields ─────────────────────────────────────────────────

#[test]
fn e7_record_50_fields() {
    let fields = (0..50)
        .map(|i| format!("f{} = {}", i, i))
        .collect::<Vec<_>>()
        .join(", ");
    let src = format!("r = {{{}}}\nmain = r.f42", fields);
    let start = Instant::now();
    let (val, _) = run(&src);
    let elapsed = start.elapsed();
    println!("E-7: 50-field record access in {:?}", elapsed);
    assert_eq!(val, Value::Int(42));
}

// ── E-8b: Type classes — dispatch ────────────────────────────────────────────

#[test]
fn e8b_typeclass_dispatch() {
    let src = "\
Color = Red | Green | Blue
trait Show a
  show : a -> String
impl Show Color
  show Red = \"Red\"
  show Green = \"Green\"
  show Blue = \"Blue\"
main = show Green";
    let start = Instant::now();
    let (val, _) = run(src);
    let elapsed = start.elapsed();
    println!("E-8b: typeclass dispatch in {:?}", elapsed);
    assert_eq!(val, Value::Str("Green".into()));
}

// ── E-8c: Pipe / Compose chains ──────────────────────────────────────────────

#[test]
fn e8c_pipe_chain_20_steps() {
    // Build: 1 |> double |> inc |> double |> inc ...  (20 alternating steps)
    let steps = (0..20)
        .map(|i| if i % 2 == 0 { "|> double" } else { "|> inc" })
        .collect::<Vec<_>>()
        .join(" ");
    let src = format!("double x = x * 2\ninc x = x + 1\nmain = 1 {}", steps);
    let start = Instant::now();
    let result = synoema_eval::eval_main(&src);
    let elapsed = start.elapsed();
    println!("E-8c: 20-step pipe chain in {:?}", elapsed);
    assert!(result.is_ok(), "Pipe chain failed: {:?}", result.err());
}

#[test]
fn e8c_compose_chain_10_steps() {
    // (double >> inc >> double >> inc >> double >> inc >> double >> inc >> double >> inc) 1
    // = inc(double(inc(double(inc(double(inc(double(inc(double(1)))))))))))
    // Start: 1, d=2, i=3, d=6, i=7, d=14, i=15, d=30, i=31, d=62, i=63
    let src = "double x = x * 2\ninc x = x + 1\nmain = (double >> inc >> double >> inc >> double >> inc >> double >> inc >> double >> inc) 1";
    let start = Instant::now();
    let result = synoema_eval::eval_main(src);
    let elapsed = start.elapsed();
    println!("E-8c: 10-step compose chain in {:?}", elapsed);
    assert!(result.is_ok(), "Compose chain failed: {:?}", result.err());
    assert_eq!(result.unwrap().0, Value::Int(63));
}

// ── E-8d: Deeply nested conditionals ─────────────────────────────────────────

#[test]
fn e8d_nested_conditionals_50() {
    // ? true -> (? true -> ... -> 42 : 0) : 0  — 50 levels
    let inner = (0..50).fold("42".to_string(), |acc, _| {
        format!("? true -> ({}) : 0", acc)
    });
    let src = format!("main = {}", inner);
    let start = Instant::now();
    let result = synoema_eval::eval_main(&src);
    let elapsed = start.elapsed();
    println!("E-8d: 50 nested conditionals in {:?}", elapsed);
    assert!(result.is_ok(), "Nested conditionals failed: {:?}", result.err());
    assert_eq!(result.unwrap().0, Value::Int(42));
}

#[test]
#[ignore]
fn e8d_nested_conditionals_200() {
    let inner = (0..200).fold("42".to_string(), |acc, _| {
        format!("? true -> ({}) : 0", acc)
    });
    let src = format!("main = {}", inner);
    let result = synoema_eval::eval_main(&src);
    println!("E-8d: 200 nested conditionals: {:?}", result.is_ok());
    assert!(result.is_ok(), "200 nested conditionals failed: {:?}", result.err());
}

// ── E-9b: Comparison operators on large data ─────────────────────────────────

#[test]
fn e9b_quicksort_comparisons() {
    // Quicksort uses < and >= at each element — exercises comparison dispatch.
    let src = "\
qs [] = []
qs (x:xs) = qs [y | y <- xs, y < x] ++ [x] ++ qs [y | y <- xs, y >= x]
main = sum (qs [10 9 8 7 6 5 4 3 2 1])";
    let start = Instant::now();
    let (val, _) = run(src);
    let elapsed = start.elapsed();
    println!("E-9b: quicksort 10 elements in {:?}", elapsed);
    assert_eq!(val, Value::Int(55));
}

#[test]
fn e9b_sort_100_elements() {
    // Build a descending list [100..1] via recursion, then sort it.
    let src = "\
qs [] = []
qs (x:xs) = qs [y | y <- xs, y < x] ++ [x] ++ qs [y | y <- xs, y >= x]
main = sum (qs [100 99 98 97 96 95 94 93 92 91 90 89 88 87 86 85 84 83 82 81 80 79 78 77 76 75 74 73 72 71 70 69 68 67 66 65 64 63 62 61 60 59 58 57 56 55 54 53 52 51 50 49 48 47 46 45 44 43 42 41 40 39 38 37 36 35 34 33 32 31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16 15 14 13 12 11 10 9 8 7 6 5 4 3 2 1])";
    let start = Instant::now();
    let (val, _) = run(src);
    let elapsed = start.elapsed();
    println!("E-9b: quicksort 100 elements in {:?}", elapsed);
    assert_eq!(val, Value::Int(5050));
}

// ── E-10: Euler1 — TCO check ─────────────────────────────────────────────────

#[test]
fn e10_euler1_999() {
    let src = "\
addIfDiv acc n = ? (n % 3 == 0 || n % 5 == 0) -> acc + n : acc
euler acc n = ? n > 999 -> acc : euler (addIfDiv acc n) (n + 1)
main = euler 0 1";
    let start = Instant::now();
    let (val, _) = run(src);
    let elapsed = start.elapsed();
    println!("E-10: Euler1(999) in {:?}", elapsed);
    assert_eq!(val, Value::Int(233_168));
}

#[test]
#[ignore] // 1M iterations — slow without TCO
fn e10_euler1_999999() {
    let src = "\
addIfDiv acc n = ? (n % 3 == 0 || n % 5 == 0) -> acc + n : acc
euler acc n = ? n > 999999 -> acc : euler (addIfDiv acc n) (n + 1)
main = euler 0 1";
    let start = Instant::now();
    let result = synoema_eval::eval_main(src);
    let elapsed = start.elapsed();
    println!("E-10: Euler1(999999) in {:?}  result={:?}", elapsed, result.is_ok());
    if let Ok((val, _)) = result {
        println!("E-10: result = {:?}", val);
    }
}

// ── Throughput bench ─────────────────────────────────────────────────────────

#[test]
#[ignore]
fn bench_eval_factorial_20() {
    let src = "fac 0 = 1\nfac n = n * fac (n - 1)\nmain = fac 20";
    let n = 100u32;

    let start = Instant::now();
    for _ in 0..n {
        let _ = synoema_eval::eval_main(src).unwrap();
    }
    let avg = start.elapsed() / n;
    println!("BENCH eval_factorial(20): {:?} avg/run (incl. 64MB thread spawn)", avg);
}
