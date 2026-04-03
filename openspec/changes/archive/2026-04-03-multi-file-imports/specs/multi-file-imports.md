---
id: multi-file-imports
type: spec
status: ready
---

# Spec: Multi-File Imports

## Syntax

```sno
import "path.sno"
```

- `import` = 1 BPE token (cl100k_base) ✓
- Path is a string literal, relative to the importing file's directory

## AST Change

```rust
pub struct ImportDecl {
    pub path: String,
    pub span: Span,
}

pub struct Program {
    pub imports: Vec<ImportDecl>,  // NEW
    pub decls: Vec<Decl>,
    pub modules: Vec<ModuleDecl>,
    pub uses: Vec<UseDecl>,
}
```

## Semantics

1. `import "path.sno"` loads and parses the target file
2. All `mod` declarations from target become available for `use`
3. Top-level functions from target are also available
4. Circular imports: detected and reported as Diagnostic error
5. Diamond imports: file loaded only once (cached by canonical absolute path)
6. Import order: imports processed before top-level declarations
7. Path resolution: relative to the importing file's directory (no absolute paths, no URLs)

## Pipeline

1. **Parser**: parse `import "path"` at top level (before mod/use/decl)
2. **Resolver** (new function in synoema-types/modules.rs or synoema-eval/lib.rs): recursively load + parse imported files, detect cycles, merge into single Program
3. **Type checker**: check merged program as one unit (unchanged)
4. **Interpreter**: eval merged program (unchanged)
5. **JIT**: compile merged program (unchanged)

## GBNF Addition

```
import_decl ::= "import" ws string_lit newline
```

## Acceptance Criteria

- [ ] `import "other.sno"` loads declarations from other file
- [ ] Imported modules accessible via `use`
- [ ] Circular imports detected and error reported
- [ ] Diamond imports: file loaded once
- [ ] Relative paths work correctly
- [ ] Works in interpreter and JIT
- [ ] GBNF grammar updated
- [ ] docs/llm/ updated
- [ ] ≥8 tests
- [ ] Example files created
