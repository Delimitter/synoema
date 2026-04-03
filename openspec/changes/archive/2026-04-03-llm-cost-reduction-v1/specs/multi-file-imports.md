---
id: multi-file-imports
type: spec
status: done
---

# Spec: Multi-File Imports

## Requirement

Add `import "path.sno"` syntax to include declarations from another file.

## Syntax

```sno
-- math.sno
mod Math
  square x = x * x
  pi = 3.14159

-- main.sno
import "math.sno"
use Math (square pi)

main = square 5
```

## BPE Verification

`import` = 1 BPE token (cl100k_base) ✓

## Semantics

- `import "path.sno"` loads and parses the target file
- All `mod` declarations from target become available for `use`
- Top-level functions from target are also available
- Path is relative to the importing file's directory
- Circular imports: detected and reported as error
- Diamond imports: file loaded only once (cached by absolute path)
- Import order: imports processed before top-level declarations

## AST Change

New field in `Program`:
```
pub struct Program {
    pub imports: Vec<ImportDecl>,  // NEW
    pub decls: Vec<Decl>,
    pub modules: Vec<ModuleDecl>,
    pub uses: Vec<UseDecl>,
}

pub struct ImportDecl {
    pub path: String,
    pub span: Span,
}
```

## Pipeline

1. **Parser**: parse `import "path"` at top level (before mod/use/decl)
2. **Resolver** (new step): recursively load + parse imported files, detect cycles, merge programs
3. **Type checker**: check merged program as one unit
4. **Interpreter**: eval merged program
5. **JIT**: compile merged program

## File Resolution

```
import "math.sno"        -- relative to current file
import "lib/utils.sno"   -- relative subdirectory
```

No package registry. No absolute paths. No URL imports. Keep it minimal.

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
- [ ] Example files: `examples/imports/main.sno` + `examples/imports/lib.sno`
