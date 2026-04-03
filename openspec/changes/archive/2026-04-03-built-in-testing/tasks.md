# Tasks: Built-in Testing

## Lexer
- [x] Add `KwTest`, `KwProp`, `KwImplies` tokens to `Token` enum
- [x] Add keyword mappings in scanner: `"test"`, `"prop"`, `"implies"`
- [x] Add `describe()`/`Display` for new tokens
- [x] Add lexer tests for new keywords

## Parser
- [x] Add `Decl::Test { name, body, span }` to AST
- [x] Add `ExprKind::Prop(Vec<String>, Box<Expr>)` to AST
- [x] Add `ExprKind::Implies(Box<Expr>, Box<Expr>)` to AST
- [x] Parse `test` declarations in `parse_program_recovering`
- [x] Parse `prop` as prefix expression in Pratt parser
- [x] Parse `implies` as infix operator (priority 3, right-assoc)
- [x] Add parser tests for test/prop/implies

## Type checker
- [x] Infer `Decl::Test` body as Bool
- [x] Infer `ExprKind::Prop` — fresh vars, body Bool, result Bool
- [x] Infer `ExprKind::Implies` — both operands Bool, result Bool
- [x] Add type checker tests (covered by parser/integration tests)

## Desugar + Core IR
- [x] Pass through `Decl::Test` (skip in desugar, no codegen)
- [x] Handle `Prop`/`Implies` in desugar (Implies → Case, Prop → pass-through body)

## Eval
- [x] Implement simple LCG random generator (no rand crate)
- [x] Implement `generate_value(ty) -> Value` for Int/Bool/String/List
- [x] Eval `ExprKind::Implies`: false→true (vacuous), true→eval body
- [x] Eval `ExprKind::Prop`: fallback eval (test runner handles generation)

## REPL runner
- [x] Extract `Decl::Test` from parsed program
- [x] Add `--filter <str>` CLI option to `synoema test`
- [x] Run test declarations after doctests
- [x] Display counterexample on prop failure
- [x] Display pass/fail summary including both doctests and tests
- [x] Type-driven value generation via `infer_prop_var_types`

## GBNF grammar
- [x] Add `test-decl` rule to grammar
- [x] Add `prop-expr` and `implies` to expression rules

## Documentation
- [x] Update `docs/llm/synoema.md` — testing section (§15)
- [x] Update `docs/specs/language_reference.md` — test/prop/implies
- [x] Update `docs/testing.md` — built-in test documentation
- [x] Update `CLAUDE.md` — test count (864)
- [x] Update `context/PROJECT_STATE.md` (864 tests)
- [x] Update `context/PHASES.md` (Phase 22)

## Examples
- [x] Create `lang/examples/testing.sno` with test + prop examples

## Verify
- [x] BPE verify: `test`, `prop`, `implies` = 1 token each (verified)
- [x] `cargo test` — 0 failures, 0 warnings (864 passed)
- [x] `synoema test examples/testing.sno` — 13/13 pass
