---
id: tasks
type: tasks
status: done
---

# Tasks: derive(Show, Eq, Ord) для ADT

## Checklist

- [x] **T1: AST — добавить derives в TypeDef**
  - `synoema-parser/src/ast.rs`: added `derives: Vec<String>` to TypeDef

- [x] **T2: Parser — parse derive clause**
  - `synoema-parser/src/parser.rs`: parse `derive (Show, Eq, Ord)` after variants

- [x] **T3: Lexer — добавить KwDerive**
  - `synoema-lexer/src/scanner.rs`: reserved `derive` as keyword
  - BPE verified: `derive` = 1 token (cl100k_base + o200k_base)
  - Note: changed from `deriving` (2 tokens) to `derive` (1 token) for BPE compliance

- [x] **T4: Type Checker — process ImplDecl methods**
  - `synoema-types/src/infer.rs`: added pass 3 to infer impl method types before Func pass

- [x] **T5: derive(Show) — no-op implementation**
  - Show is a builtin that already handles all types including ADTs
  - derive(Show) is accepted syntactically but generates no equations

- [x] **T6: derive(Eq) — structural equality**
  - `synoema-parser/src/derive.rs`: generates `eq _x _y = _x == _y`
  - Delegates to built-in `==` which handles structural ADT equality

- [x] **T7: derive(Ord) — variant index comparison**
  - `synoema-parser/src/derive.rs`: generates `cmp` using variant index chain
  - Maps each variant to its declaration index, then compares: -1/0/1

- [x] **T8: Error handling**
  - Unknown derive trait → ParseError with span
  - derive on empty ADT → no-op (graceful)

- [x] **T9: Tests — 21 new tests**
  - Parser: 4 tests (single trait, multiple traits, no derive, unknown trait error)
  - Eval: 17 tests (show enum, show with fields, eq same/diff/constructors, ord ordering, etc.)
  - Total: 842/842 tests pass, 0 failures

- [x] **T10: Documentation**
  - `docs/llm/synoema.md` — added derive example
  - `docs/specs/language_reference.md` — updated EBNF grammar, added derive clause
  - `tools/constrained/synoema.gbnf` — updated type-def rule with derive
  - `lang/examples/derive.sno` — complete example

- [x] **T11: BPE verification**
  - `derive` = 1 BPE token (cl100k_base) ✓
  - Added to `tools/bpe-verify/verify_bpe.py`

- [x] **T12: Final verification**
  - `cargo test`: 842 tests, 0 failures, 0 warnings
  - Interpreter: all derives work correctly
  - JIT: Show and Eq work; Ord has pre-existing JIT limitation with nested ADT conditionals
  - Manual impl overrides derive correctly
  - Also fixed: structural Con equality in JIT (`synoema_val_eq`)
  - Also fixed: missing `dealloc` import in `synoema-codegen/src/runtime.rs`
