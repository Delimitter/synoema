---
id: tasks
type: tasks
status: done
---

# Tasks: Memory Management v2

## Checklist

- [x] **T1: fd_open — файловое чтение (interpreter)**
  - `synoema-eval/src/eval.rs` — added `fd_open` builtin
  - Uses `File::open` → `BufReader` → `IO_READERS` map
  - Existing `fd_readline`, `fd_close` work without changes

- [x] **T2: fd_open_write — файловая запись (interpreter)**
  - `synoema-eval/src/eval.rs` — added `fd_open_write` builtin
  - Uses `File::create` → `BufWriter` → `IO_WRITERS` map
  - Existing `fd_write`, `fd_close` work without changes

- [x] **T3: fd_open / fd_open_write — JIT FFI**
  - Deferred: JIT has no fd_* builtins at all (fd_popen etc are interpreter-only)
  - Adding file I/O to JIT requires thread-local IO maps in runtime.rs (separate task)
  - Not a blocker: stress_server and file processing run via interpreter

- [x] **T4: Arena overflow tracking**
  - `synoema-codegen/src/runtime.rs` — added `overflow_allocs: Vec<(*mut u8, Layout)>` and `overflow_warned: bool` to Arena
  - On overflow: `eprintln!` warning (once per cycle) + tracked allocation

- [x] **T5: Arena overflow cleanup**
  - `synoema-codegen/src/runtime.rs` — `reset()` now iterates + `dealloc()` all tracked overflow allocations

- [x] **T6: arena_save / arena_restore**
  - `synoema-codegen/src/runtime.rs` — `pub extern "C" fn arena_save() -> i64` and `arena_restore(saved: i64)`
  - Safety check: restore only if saved <= current offset

- [x] **T7: Type signatures**
  - `synoema-types/src/infer.rs` — `fd_open : String -> Int`, `fd_open_write : String -> Int`

- [x] **T8: Example — streaming file processing**
  - `lang/examples/file_stream.sno` — reads factorial.sno line-by-line

- [x] **T9: Tests (9 new tests)**
  - eval: fd_open_read_line, fd_open_read_multiple_lines, fd_open_missing_file_is_err
  - eval: fd_open_write_creates_file, fd_open_write_then_read, fd_open_write_missing_dir_is_err
  - codegen: arena_save_restore_basic, arena_save_restore_preserves_offset, arena_overflow_tracked_cleanup

- [x] **T10: Documentation (rule 7a)**
  - `docs/llm/stdlib.md` — added fd_open, fd_open_write to I/O table
  - `context/PROJECT_STATE.md` — updated with arena hardening + streaming I/O

- [x] **T11: BPE verification**
  - `fd_open` / `fd_open_write` are builtin names, not operators → no BPE alignment needed
  - `arena_save` / `arena_restore` are internal runtime FFI → not in GBNF

- [x] **T12: Final verification**
  - `cargo test` — 806 tests pass, 0 failures, 0 warnings (excluding pre-existing parser warnings)
  - `cargo build` — 0 new warnings
  - Example file_stream.sno runs correctly in interpreter
