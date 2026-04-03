---
id: design
type: design
status: draft
---

# Design: Multi-File Imports

## Approach: Merged Program

Импорты разрешаются **до type checking** — все файлы мержатся в единый `Program`, который проходит через существующий pipeline без изменений.

```
    main.sno
    ┌──────────────────┐
    │ import "math.sno" │
    │ import "utils.sno"│
    │ main = ...        │
    └────────┬─────────┘
             │
             ▼ resolve_imports()
             │
    ┌────────┴─────────┐
    │  Merged Program   │
    │  (all decls from  │
    │   all files)      │
    └────────┬─────────┘
             │
             ▼
    Type Check → Eval/JIT (unchanged)
```

### Import Resolution Algorithm

```rust
fn resolve_imports(
    entry_path: &Path,
    source: &str,
) -> Result<Program, Vec<Diagnostic>> {
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut merged = Program::empty();

    resolve_recursive(entry_path, source, &mut visited, &mut merged)?;
    Ok(merged)
}

fn resolve_recursive(
    file_path: &Path,
    source: &str,
    visited: &mut HashSet<PathBuf>,
    merged: &mut Program,
) -> Result<(), Vec<Diagnostic>> {
    let canonical = file_path.canonicalize()?;

    // Diamond caching
    if visited.contains(&canonical) {
        return Ok(());
    }
    visited.insert(canonical.clone());

    let program = parse(source)?;

    // Process imports first
    for import in &program.imports {
        let import_path = file_path.parent()
            .unwrap()
            .join(&import.path);

        // Cycle detection
        if visited.contains(&import_path.canonicalize()?) {
            // Already processed (diamond) — skip
            continue;
        }

        let import_source = std::fs::read_to_string(&import_path)?;
        resolve_recursive(&import_path, &import_source, visited, merged)?;
    }

    // Merge declarations
    merged.decls.extend(program.decls);
    merged.modules.extend(program.modules);
    merged.uses.extend(program.uses);

    Ok(())
}
```

### Cycle Detection

```
    a.sno ──import──▶ b.sno ──import──▶ a.sno
                                         ↑
                                    Already in visited set!
                                    → Diagnostic error
```

Различие cycle vs diamond:
- **Diamond**: `a→b, a→c, b→d, c→d` — `d` в visited, но не в current stack → OK, skip
- **Cycle**: `a→b→a` — `a` в current call stack → ERROR

Для корректного различения нужен **отдельный** `in_progress: HashSet` помимо `visited`.

### AST Changes

```rust
// synoema-parser/src/ast.rs
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

### Parser Changes

```rust
// At top of file, before any decl:
// import "path.sno"
fn parse_import(&mut self) -> Result<ImportDecl, ParseError> {
    self.expect(Token::KwImport)?;
    let path = self.expect_string()?;
    Ok(ImportDecl { path, span: ... })
}
```

### Resolver Location

New function in `synoema-eval/src/lib.rs` or new `synoema-eval/src/resolve.rs`.
Called from REPL before type checking.

### GBNF Grammar

```
import_decl ::= "import" ws string_lit newline
program ::= (import_decl)* (decl | module | use)*
```

### Error Cases

| Situation | Error Code | Message |
|-----------|-----------|---------|
| File not found | `import_not_found` | `cannot find "path.sno"` |
| Circular import | `import_cycle` | `circular import: a.sno → b.sno → a.sno` |
| Parse error in imported file | pass-through | Original parse error with file context |
| Type error in imported decls | pass-through | Original type error with file context |
