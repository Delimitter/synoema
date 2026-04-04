# Design: Build Command for Synoema Projects

## Architecture

### Command Structure

```
synoema build [OPTIONS] <FILE>

Options:
  --output, -o <PATH>     Output file path (default: <FILE>.bc)
  --check                  Perform type checking only, no codegen
  --verbose, -v            Show compilation phases and diagnostics
  --errors <FORMAT>        Output format: human (default) or json
```

### Implementation Strategy

#### 1. CLI Integration (`synoema-repl`)

Add a new `Build` command variant to the existing command parser:

```rust
pub enum Command {
    Init { project_name: String },
    Run { file: PathBuf },
    Jit { file: PathBuf },
    Eval { expr: String },
    Build { file: PathBuf, output: Option<PathBuf>, check_only: bool },
    Test { dir: PathBuf },
    Doc { file: PathBuf },
}
```

#### 2. Build Pipeline (Compiler Phases)

Reuse existing compiler phases in sequence:

1. **Lexical Analysis** (synoema-lexer) → tokens
2. **Parsing** (synoema-parser) → AST
3. **Type Checking** (synoema-types) → typed AST + constraints
4. **Desugaring** (synoema-core) → Core IR
5. **Optimization** (synoema-core) → optimized Core IR
6. **Code Generation** (synoema-codegen) → CLIF IR
7. **Serialization** → bytecode file (`.bc`)

#### 3. Output Format

**Bytecode format** (`.sno.bc`):

```
[HEADER: 4 bytes magic "SNOE"]
[VERSION: 1 byte, e.g. 0x01]
[METADATA: serialized type info, imports, etc.]
[CODE: serialized CLIF IR or Core IR]
[CHECKSUM: 4 bytes CRC32]
```

For now, output the textual representation of Core IR (intermediate step). Future phases can serialize to binary.

#### 4. Error Handling

Use existing `synoema-diagnostic` crate:
- Collect all errors in one pass (error recovery)
- Format output as human-readable or JSON
- Exit with code 1 if errors, 0 if success

#### 5. Incremental Builds

**Phase 1 (MVP)**: No caching. Each `build` compiles from source.

**Future optimization**: Track modification times and skip unchanged files (requires manifest file).

## Key Design Decisions

1. **Reuse existing compiler** — no new compilation logic, just wire together existing phases
2. **Output to .bc file** — easy to identify built artifacts vs source
3. **Textual Core IR output** — simpler than binary serialization for MVP, still useful for inspection
4. **Error recovery via diagnostic system** — consistent with other subcommands
5. **No incremental builds in Phase 1** — simplifies implementation, future feature

## Dependencies & Integration

- `synoema-repl`: CLI parsing + main build orchestration
- `synoema-lexer`, `synoema-parser`, `synoema-types`, `synoema-core`, `synoema-codegen`: existing phases
- `synoema-diagnostic`: error reporting
- Standard library: `std::fs`, `std::path::PathBuf`

## Testing Strategy

- Unit tests in `synoema-repl` for CLI parsing
- Integration tests: build example files, verify `.bc` output exists
- Error cases: invalid syntax, type errors, missing files
- Edge cases: empty files, files with only comments
