# Tasks: formatter

## Checklist

- [x] T1: Create `lang/crates/synoema-repl/src/fmt.rs` — `format_source(source) -> Result<String, Diagnostic>` with formatting rules
- [x] T2: Add `format_file(path, check) -> Result<bool, ...>` — single file format/check
- [x] T3: Add `format_directory(path, check) -> Result<(usize, usize), ...>` — recursive `.sno` file walk
- [x] T4: Add `Some("fmt")` subcommand in `main.rs` — wire up `fmt <file|dir> [--check]`
- [x] T5: Update `--help` output in `main.rs` — add `synoema fmt` usage
- [x] T6: Tests — idempotency: format(format(code)) == format(code) for function defs, patterns, ADTs
- [x] T7: Tests — comment preservation: `--` and `---` comments survive formatting
- [x] T8: Tests — tab replacement, trailing whitespace removal, blank line collapse, final newline
- [x] T9: Tests — malformed code returns error (not panic)
- [x] T10: Verify all existing examples compile (examples use canonical formatting already)
- [x] T11: `cargo test` — 0 failures, 0 build warnings
