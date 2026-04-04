# Specification: Build Command

## 1. CLI Interface

### Command Syntax

```
synoema build [OPTIONS] <FILE>
```

### Arguments

| Argument | Type | Description |
|----------|------|-------------|
| `<FILE>` | path | Path to the `.sno` file to build |

### Options

| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--output` | `-o` | path | `<FILE>.bc` | Output file path for bytecode |
| `--check` | - | flag | false | Perform type checking only, skip codegen |
| `--verbose` | `-v` | flag | false | Show detailed compilation steps |
| `--errors` | - | string | `human` | Output format: `human` or `json` |

### Exit Codes

- `0`: Build successful
- `1`: Compilation error (syntax, type, or codegen)
- `2`: I/O error (file not found, write permission denied)

## 2. Behavior

### Successful Build

```bash
$ synoema build examples/quicksort.sno
Built: examples/quicksort.sno.bc (42 KB)
```

### Type Check Only

```bash
$ synoema build --check examples/quicksort.sno
Type check passed
```

### Compilation Error

```bash
$ synoema build examples/invalid.sno
Error [E0001]: Type mismatch in function 'sort'
  → examples/invalid.sno:5:10
  5 | let x: Int = "string"
      |           ^^^^^^^^^ expected Int, found String

  hint: Try wrapping with show/to_string
exit 1
```

### JSON Output

```bash
$ synoema build --errors json examples/quicksort.sno
{"status": "ok", "output": "examples/quicksort.sno.bc", "size_bytes": 42000}

$ synoema build --errors json examples/invalid.sno
{"status": "error", "errors": [{"code": "E0001", "message": "Type mismatch", "span": {...}}]}
```

## 3. Output Format

### Bytecode File (`.sno.bc`)

**Textual format** (v1):

```
SYNOEMA BYTECODE v1
source: examples/quicksort.sno
type-signature: [Int] -> [Int]
dependencies: []

[Core IR]
let sort =
  fix sort = \xs ->
    ? null xs -> xs
    : let pivot = head xs
      let rest = tail xs
      let left = [x | x <- rest, x < pivot]
      let right = [x | x <- rest, x >= pivot]
      ++ (sort left) [pivot] (sort right)
```

**Future format** (v2): Binary serialization with:
- 4-byte magic header `SNOE`
- 1-byte version
- Serialized type information
- Serialized Core IR bytecode
- 4-byte CRC32 checksum

## 4. Build Process

### Step 1: Parse Arguments

Extract file path and options. Validate:
- File exists and is readable
- Output path is writable (or directory exists)

### Step 2: Compile

Run through compiler pipeline:
1. Tokenize (synoema-lexer)
2. Parse (synoema-parser)
3. Type check (synoema-types)
4. Desugar (synoema-core)
5. Optimize (synoema-core)
6. Codegen to Core IR (synoema-codegen)

Collect all diagnostics (errors + warnings).

### Step 3: Handle Results

- **If errors**: Report with context, exit 1
- **If check_only**: Report "Type check passed", exit 0
- **If success**: Generate bytecode, write to output file, report success

### Step 4: Output Summary

Print:
```
Built: <output_path> (<size> bytes)
Warnings: <count> (if any)
```

## 5. Error Recovery

Use error recovery mechanisms from synoema-diagnostic:
- Collect all parsing errors in one pass
- Continue type checking where possible
- Report up to N errors (N = 10 default)
- Suggest fixes based on error type (existing in diagnostic crate)

## 6. Performance

- **No optimization for build time in Phase 1** (correctness first)
- Full compilation for each invocation (no incremental tracking)
- Expected compile time: ~100ms per example file

## 7. Future Extensions

1. **Incremental builds** — track file mtimes, skip unchanged files
2. **Parallel compilation** — multi-threaded for large projects
3. **Binary output** — native code via JIT in `--native` mode
4. **Bytecode caching** — serialize/deserialize with versioning
5. **Dependency management** — package manifest (separate feature)
