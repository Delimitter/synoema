# Design: Multi-File Imports

## Architecture Decision

The import resolver merges all imported files into a **single flat Program** before any downstream processing. This means type checker, evaluator, and JIT compiler require **zero changes** — they already work with flattened programs after `resolve_modules()`.

## Implementation Plan

### 1. Parser Changes (synoema-parser)

**Token:** `import` keyword already needs to be added to the lexer as a keyword token.

**AST:** Add `ImportDecl` struct and `imports: Vec<ImportDecl>` to `Program`.

**Parsing:** At top level, when `import` token is seen, parse `import "string_literal"` and collect into `program.imports`.

### 2. Import Resolver (new function)

**Location:** `synoema-eval/src/lib.rs` (shared between interpreter and JIT entry points) — a new `resolve_imports()` function.

**Algorithm:**
```
resolve_imports(program, base_dir) -> Result<Program, Diagnostic>:
    seen = HashSet<PathBuf>       // canonical paths already loaded (diamond)
    stack = Vec<PathBuf>          // current import chain (cycle detection)
    merged = Program::empty()

    fn resolve_recursive(path, seen, stack, merged):
        canonical = canonicalize(base_dir / path)
        if canonical in stack: return Err(circular import diagnostic)
        if canonical in seen: return Ok(())  // diamond: skip
        seen.insert(canonical)
        stack.push(canonical)

        source = read_file(canonical)?
        program = parse(source)?

        // First resolve this file's imports recursively
        for imp in program.imports:
            resolve_recursive(imp.path, seen, stack, merged)

        // Then add this file's declarations to merged
        merged.modules.extend(program.modules)
        merged.decls.extend(program.decls)
        merged.uses.extend(program.uses)

        stack.pop()

    // Add root file's imports
    for imp in program.imports:
        resolve_recursive(imp.path, seen, stack, merged)

    // Add root file's own declarations AFTER imports
    merged.modules.extend(program.modules)
    merged.decls.extend(program.decls)
    merged.uses.extend(program.uses)

    return Ok(merged)
```

### 3. Integration Points

- `synoema_eval::eval_main_inner()`: call `resolve_imports()` before `resolve_modules()`
- `synoema_codegen::compile_and_run()`: call `resolve_imports()` before `resolve_modules()`
- `synoema_repl::run_file()` / `jit_file()`: pass file directory as `base_dir`
- REPL `:load`: pass current working directory as `base_dir`

### 4. Diagnostic Error

New error variant for circular import:
```
error[E_IMPORT_CYCLE]: circular import detected
  --> main.sno:1:1
  |
1 | import "a.sno"
  | ^^^^^^^^^^^^^^ a.sno → b.sno → main.sno
```

### 5. File Not Found

```
error[E_IMPORT_NOT_FOUND]: file not found
  --> main.sno:1:1
  |
1 | import "missing.sno"
  | ^^^^^^^^^^^^^^^^^^^^ file not found: /path/to/missing.sno
```

## What Does NOT Change

- Type checker (works on merged flat program)
- Evaluator (works on merged flat program)
- JIT compiler (works on merged flat program)
- Core IR / desugaring (works on flat program)
- Module resolution (works on flat program)

## Dependencies

- No new crate dependencies (std::fs::read_to_string, std::path)
- `import` is already 1 BPE token ✓
