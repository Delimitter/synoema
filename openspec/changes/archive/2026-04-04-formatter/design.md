# Design: Code Formatter

## Architecture

### Why source-text-based (not AST-based)

The Synoema parser is lossy in 3 ways:
1. **String interpolation** desugared: `"${x}"` → `"" ++ show x ++ ""`
2. **Record punning** desugared: `{x}` → `{x = x}`
3. **Regular comments** (`--`) stripped by lexer

An AST-based pretty printer would destroy these constructs. The formatter must operate on source text and use the AST only for validation.

### Formatter pipeline

```
source text → parse (validate) → apply formatting rules → output
                    ↓ (on error)
              diagnostic → exit 1
```

### Formatting rules (applied in order)

1. **Tab replacement**: `\t` → `  ` (2 spaces)
2. **Trailing whitespace**: remove from each line
3. **Blank line normalization**: collapse 2+ consecutive blank lines to 1
4. **Final newline**: ensure file ends with exactly 1 `\n`

These rules are safe (never change semantics), order-independent (applying them in any order produces the same result), and idempotent.

## Implementation

### New file: `lang/crates/synoema-repl/src/fmt.rs`

```rust
pub fn format_source(source: &str) -> Result<String, Diagnostic> {
    // 1. Validate: parse to check for syntax errors
    synoema_parser::parse(source).map_err(|e| ...)?;

    // 2. Apply formatting rules to source text
    Ok(apply_rules(source))
}

fn apply_rules(source: &str) -> String {
    let mut lines: Vec<String> = source.lines().map(|l| {
        // Tab → 2 spaces, trim trailing whitespace
        l.replace('\t', "  ").trim_end().to_string()
    }).collect();

    // Collapse consecutive blank lines
    // ... dedup logic ...

    // Ensure trailing newline
    result.push('\n');
    result
}

pub fn format_file(path: &Path, check: bool) -> Result<bool, ...> {
    let source = fs::read_to_string(path)?;
    let formatted = format_source(&source)?;
    if check {
        Ok(source == formatted)
    } else {
        if source != formatted {
            fs::write(path, &formatted)?;
        }
        Ok(true)
    }
}
```

### CLI integration in `main.rs`

```rust
Some("fmt") => {
    let path = positional.get(1).unwrap_or_else(|| ...);
    let check = positional.iter().any(|a| *a == "--check");
    // If path is directory → walk recursively for .sno files
    // If file → format single file
}
```

## Files Changed

| File | Change |
|------|--------|
| `repl/src/fmt.rs` | NEW — formatting logic |
| `repl/src/main.rs` | +`Some("fmt")` subcommand, +`--help` update |
| `eval/src/tests.rs` | +formatter tests (idempotency, comments, errors) |

## Testing Strategy

Tests go in `eval/src/tests.rs` (or inline in `fmt.rs`) since the formatter uses the parser for validation. Test cases:

1. **Idempotency**: `format(format(code)) == format(code)` for various inputs
2. **Comment preservation**: `-- comment` and `--- doc comment` survive formatting
3. **Tab replacement**: tabs → 2 spaces
4. **Trailing whitespace**: removed
5. **Blank lines**: 2+ → 1
6. **Final newline**: always present
7. **Malformed code**: returns error, doesn't modify
8. **Empty file**: handled gracefully
