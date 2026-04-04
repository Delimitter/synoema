# Tasks: Build Command Implementation

## Phase A: CLI Integration

- [x] A1. Add `Build` variant to `Command` enum in `synoema-repl/src/main.rs`
  - Include fields: `file: PathBuf`, `output: Option<PathBuf>`, `check_only: bool`, `verbose: bool`, `errors_format: String`

- [x] A2. Extend CLI argument parser in `synoema-repl/src/main.rs`
  - Parse `synoema build <FILE>`
  - Parse `--output` / `-o` option
  - Parse `--check` flag
  - Parse `--verbose` / `-v` flag
  - Parse `--errors json|human` option
  - Validate file exists before dispatching

- [x] A3. Add help text for `build` command
  - Update `--help` output to include `build` in command list
  - Show usage examples: `synoema build file.sno`, `synoema build -o out.bc file.sno`

## Phase B: Build Pipeline Implementation

- [x] B1. Create `BuildConfig` struct in `synoema-repl/src/main.rs`
  - Fields: `source_path`, `output_path`, `check_only`, `verbose`, `errors_format`
  - Implement validation (file readable, output writable)

- [x] B2. Implement `build_file()` function orchestrating compiler phases
  - Takes `BuildConfig`
  - Calls lexer → parser → type checker → desugar → optimize → codegen in sequence
  - Collects diagnostics from each phase
  - Returns `Result<CompiledModule, Vec<Diagnostic>>`

- [x] B3. Integrate `synoema-diagnostic` for error reporting
  - Use existing error formatter (human or JSON)
  - Pass diagnostics to formatter based on `--errors` option
  - Map diagnostic severity (error/warning/info) to output

- [x] B4. Handle `--check` mode
  - Skip codegen and output generation if `--check` is set
  - Return success message if type checking passes

## Phase C: Output Generation

- [x] C1. Implement bytecode serialization (v1: textual Core IR)
  - Create `serialize_bytecode()` function
  - Write header: `SYNOEMA BYTECODE v1`
  - Write metadata: source file, type signature, imports
  - Write Core IR in textual form
  - Write footer

- [x] C2. Write bytecode to file
  - Create output directory if needed
  - Write to `output_path` specified in `BuildConfig`
  - Set file permissions to 0644 (readable)
  - Report file size and path on success

- [x] C3. Add output summary messaging
  - Success: `Built: <path> (<size> bytes)`
  - Warnings: `Warnings: <count>`
  - Time: `(compiled in <ms> ms)` (optional, if verbose)

## Phase D: Error Handling & Validation

- [x] D1. Validate input file
  - Check file exists (exit code 2)
  - Check file is readable (exit code 2)
  - Check file has `.sno` extension (warning or error?)

- [x] D2. Validate output path
  - Ensure directory is writable (exit code 2)
  - Handle write failures gracefully (exit code 2)

- [x] D3. Handle compilation errors
  - Exit code 1 if any error-level diagnostic
  - Still report warnings
  - Do not create output file on error

- [x] D4. Error recovery (use existing infrastructure)
  - Call `parse_recovering()` instead of `parse()`
  - Call `typecheck_recovering()` instead of `typecheck()`
  - Report up to 10 errors per phase

## Phase E: Testing

- [x] E1. Unit tests for CLI parsing
  - Test valid build command: `["synoema", "build", "test.sno"]`
  - Test with `-o` option: `["synoema", "build", "-o", "out.bc", "test.sno"]`
  - Test with `--check`: `["synoema", "build", "--check", "test.sno"]`
  - Test invalid file (should error)

- [x] E2. Integration tests for build output
  - Build `examples/quicksort.sno` → verify `.bc` file exists
  - Build `examples/factorial.sno` → verify bytecode contains correct Core IR
  - Test error case: build file with type error → verify error output
  - Test error case: build non-existent file → exit code 2

- [x] E3. Regression tests
  - Run full test suite: `cargo test` (should pass 100%)
  - Verify no new warnings introduced
  - Check that existing `run`, `jit`, `eval`, `test` commands still work

- [x] E4. Edge cases
  - Empty file: should parse successfully
  - File with only comments: should succeed
  - File with syntax error: should report error with context
  - File with type error: should report error with suggestion

## Phase F: Documentation & Polish

- [x] F1. Update CLAUDE.md
  - Add `cargo run -p synoema-repl -- build examples/quicksort.sno` to command reference

- [ ] F2. Create build command examples
  - Document basic usage, `--check` mode, error examples
  - Add to `docs/user/README.md` or create `docs/user/build.md`

- [x] F3. Finalize exit codes
  - Document: 0 = success, 1 = compile error, 2 = I/O error
  - Ensure consistent across all error paths

- [ ] F4. Code review checklist
  - Check RULES.md compliance (BPE tokens, naming, etc.)
  - Verify all error messages follow diagnostic format
  - Ensure no compiler warnings

---

## Dependency Chain

1. **A1 → A2 → A3** (CLI setup)
2. **B1 → B2 → B3 → B4** (Build pipeline, depends on A)
3. **C1 → C2 → C3** (Output, depends on B)
4. **D1 → D2 → D3 → D4** (Error handling, independent but parallel with C)
5. **E1 → E2 → E3 → E4** (Testing, depends on C+D)
6. **F1 → F2 → F3 → F4** (Documentation, final phase)

**Critical path**: A → B → C → E (implement, test, done)

**Estimated effort**: ~4-6 hours for experienced Rust developer
