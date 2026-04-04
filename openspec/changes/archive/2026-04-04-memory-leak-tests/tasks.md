# Tasks: Memory Leak Tests

## Introspection API
- [x] Add `arena_offset() -> usize` to runtime.rs (pub fn, reads ARENA offset)
- [x] Add `arena_overflow_count() -> usize` to runtime.rs (pub fn, reads overflow_allocs.len())
- [x] Add `arena_region_depth() -> usize` to runtime.rs (pub fn, reads region_depth)
- [x] Export new functions in lib.rs

## M-1: Arena Leak Detection Tests
- [x] Test: arena_reset sets offset to 0
- [x] Test: arena_reset clears overflow allocs (count == 0)
- [x] Test: arena_reset resets region_depth to 0
- [x] Test: compile_and_run leaves arena clean (offset == 0 after)

## M-2: Region Balance Tests
- [x] Test: region_enter/region_exit pair restores offset
- [x] Test: nested regions (depth 3) — correct enter/exit balance
- [x] Test: region_exit at depth 0 is no-op (no underflow)
- [x] Test: JIT program with regions — region_depth == 0 after run

## M-3: Leak Audit Existing Tests
- [x] Audit all non-ignored JIT tests: add arena_offset == 0 assertion after each
- [x] Fix any discovered leaks (none found — all 23 programs clean)

## M-4: Stress Cycles
- [x] Test: 1000 alloc-reset cycles — offset stable at 0
- [x] Test: repeated overflow-reset cycles — overflow_count stable at 0

## Finalize
- [x] `cargo test -p synoema-codegen` — 0 failures, 0 warnings
- [x] Update test count in CLAUDE.md (864→875, 880+→890+)
