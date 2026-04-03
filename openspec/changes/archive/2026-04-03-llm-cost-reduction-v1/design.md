---
id: design
type: design
status: done
---

# Design: LLM Cost Reduction v1

## Implementation Order

Ordered by dependency + risk (lowest risk first):

1. **Stdlib catalog** — docs only, zero code risk
2. **Type aliases** — parser + type checker, no runtime changes
3. **Error recovery** — parser + type checker refactor, no new syntax
4. **String interpolation** — lexer + parser + desugar, runtime via existing `++` and `show`
5. **Multi-file imports** — new resolver step, touches all pipeline stages

## Technical Decisions

### D1: Type alias = textual substitution

Type aliases are expanded BEFORE type inference. No `TypeAlias` in Core IR.
Rationale: simplest implementation, no runtime cost, type errors show expanded form.

Expansion phase: new function `expand_aliases(program: &Program) -> Program` in `synoema-types`.
- Collects all `Decl::TypeAlias` into `HashMap<String, (Vec<String>, TypeExpr)>`
- Walks all `TypeExpr` nodes, replaces `TypeExprKind::Con(name)` where name is in alias map
- Parametric: substitute type args positionally

### D2: Error recovery via skip-to-next-decl

Parser recovery: on error, skip tokens until next top-level declaration boundary.
Detection: token at column 0 that starts a valid declaration (lowercase ident followed by pattern/`=`, or `mod`/`use`/`type`/`trait`/`impl`).

Type checker recovery: on error, assign `Type::Var(fresh)` and continue.
Failed declarations get error-typed but don't block subsequent inference.

### D3: String interpolation desugars to `++` chains

No new runtime primitive. `"a ${e} b"` → `"a " ++ show e ++ " b"`.
Lexer handles `${...}` detection by brace-counting inside strings.

Corner case: nested braces `${f {x=1}}` — brace counter tracks depth.

### D4: Multi-file imports via recursive resolver

New module in `synoema-parser` or standalone: `resolve_imports(path, parsed) -> MergedProgram`.
- Recursive with visited-set for cycle detection
- Cache parsed files by canonical path
- Merge: concatenate modules/decls/uses, imports stripped

### D5: GBNF updates

New rules to add:
```
type_alias ::= "type" UNAME (LNAME)* "=" type_expr NL
import_decl ::= "import" STRING NL
string_interp ::= '"' (CHAR | "${" expr "}")* '"'
```

### D6: Documentation updates

After each feature:
- `docs/llm/synoema.md` — index update
- `docs/llm/types.md` — add type alias syntax
- `docs/llm/stdlib.md` — new file (task 1)
- `docs/llm/modules.md` — add import syntax
- `docs/specs/language_reference.md` — formal spec
- `context/PROJECT_STATE.md` — status update
- `CLAUDE.md` — test count update

## Affected Crates

| Feature | lexer | parser | types | core | eval | codegen | diagnostic |
|---------|-------|--------|-------|------|------|---------|------------|
| Stdlib catalog | — | — | — | — | — | — | — |
| Type alias | — | ✓ | ✓ | — | — | — | — |
| Error recovery | — | ✓ | ✓ | — | — | — | ✓ |
| String interp | ✓ | ✓ | — | ✓ | — | — | — |
| Multi-file import | — | ✓ | — | — | ✓ | ✓ | — |

## Risks

- **String interpolation lexer complexity**: brace counting inside strings adds state to lexer. Mitigate: thorough edge-case tests.
- **Error recovery may change existing error messages**: regression risk. Mitigate: keep existing single-error tests passing.
- **Multi-file imports + JIT**: JIT compiles one program; merged program must work. Mitigate: merge happens before JIT sees program.
