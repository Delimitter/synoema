# Design: Fix Benchmark Suite

## Approach

1. Build release binary: `cd lang && cargo build --release -p synoema-repl`
2. Verify binary exists at expected path
3. Run Phase A: `cargo run --manifest-path benchmarks/runner/Cargo.toml -- run --phases token`
4. Run Phase B: `cargo run --manifest-path benchmarks/runner/Cargo.toml -- run --phases runtime -v`
5. Parse results from `benchmarks/results/<latest>/raw.json`
6. Fill placeholder data in articles #8, #9, #11

## Phase B Stability Fix

Runner looks for binary at:
- `lang/target/release/synoema` (preferred)
- `lang/target/release/synoema-repl` (fallback)

If neither exists, it builds release. But if debug binary exists, it uses debug → slow.

Solution: ensure release binary exists BEFORE running Phase B.

## Data Flow

```
Phase A (token_count.py) → raw.json → fill #8 tables
Phase B (runtime.rs)     → raw.json → fill #9 tables + #8 "attention compute" section
Phase A totals           →            fill #11 cost calculations
```
