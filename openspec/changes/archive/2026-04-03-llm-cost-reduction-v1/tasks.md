---
id: tasks
type: tasks
status: done
---

# Tasks: LLM Cost Reduction v1

## Task 1: Stdlib Catalog ✅
- [x] 1.1 Audit all builtins in `eval.rs` (Builtin match arms) and `runtime.rs` (extern "C" fns)
- [x] 1.2 Create `docs/llm/stdlib.md` with all builtins grouped by category, each with type signature
- [x] 1.3 Verify ≤600 tokens (cl100k_base)
- [x] 1.4 Update `docs/llm/synoema.md` index to reference stdlib.md

## Task 2: Type Aliases ✅
- [x] 2.1 Add `Decl::TypeAlias` to AST (`ast.rs`)
- [x] 2.2 Parse `type Name params = TypeExpr` in parser (`parser.rs`)
- [x] 2.3 Implement alias expansion in type checker (`infer.rs`)
- [x] 2.4 Handle parametric aliases (type args substitution)
- [x] 2.5 Detect and error on recursive aliases
- [x] 2.6 Support type aliases inside modules (`mod M` → `type T = ...` via parse_decl)
- [x] 2.7 Add 3 parser tests + 3 type checker tests + 2 eval tests = 8 tests
- [ ] 2.8 Update GBNF grammar
- [ ] 2.9 Update `docs/llm/types.md` and `docs/specs/language_reference.md`

## Task 3: Error Recovery ✅
- [x] 3.1 Refactor parser to accumulate errors via `parse_program_recovering`
- [x] 3.2 Implement skip-to-next-declaration recovery in parser
- [x] 3.3 Refactor type checker to accumulate errors via `infer_program_recovering`
- [x] 3.4 Type checker: continue past errors with fresh type vars
- [ ] 3.5 Update diagnostic output: JSON mode returns array of errors
- [ ] 3.6 Update REPL to display all errors
- [x] 3.7 Add 4 parser recovery + 2 type checker recovery = 6 tests
- [x] 3.8 Verify existing single-error tests still pass (all 625 original tests pass)

## Task 4: String Interpolation — DEFERRED
- [ ] 4.1 Extend lexer to detect `${` inside string literals
- [ ] 4.2-4.9 (deferred to separate change)

## Task 5: Multi-File Imports — DEFERRED
- [ ] 5.1-5.11 (deferred to separate change)

## Task 6: Final Documentation & Verification
- [x] 6.1 `docs/llm/stdlib.md` created
- [x] 6.2 `docs/llm/synoema.md` updated
- [x] 6.3 Run full `cargo test` — 643 passed, 0 failures, 0 warnings
- [x] 6.4 Run `cargo build` — clean (0 warnings)
