# Proposal: Code Formatter (`synoema fmt`)

## Problem Statement

Synoema has no canonical formatter. LLM-generated code may have inconsistent indentation, trailing whitespace, or irregular blank line spacing. A formatter provides:
- Canonical code style for LLM-generated and human-written code
- `--check` mode for CI/validation pipelines
- Batch formatting of entire directories

## Design Constraints

The Synoema parser is **lossy** — string interpolation (`"${expr}"`) is desugared to `show` + `++`, and record punning (`{x}`) is desugared to `{x = x}`. A pure AST-based pretty printer would destroy these user-written constructs. Regular `--` comments are also stripped by the lexer (only `---` doc comments survive to AST).

Therefore: the formatter operates on **source text**, not AST output. The AST is used only to **validate** that the code parses successfully before formatting.

## Scope

### Source-level formatting rules
- Replace tabs with 2 spaces
- Remove trailing whitespace from each line
- Collapse 2+ consecutive blank lines to 1 blank line
- Ensure file ends with exactly 1 newline
- Preserve all comments (`--` and `---`) and string content

### CLI integration
- `synoema fmt <file.sno>` — format in-place
- `synoema fmt <dir/>` — format all `.sno` files recursively
- `synoema fmt --check <file.sno>` — exit 1 if not formatted (no modification)

### Validation
- Parse the source before formatting — reject malformed code with diagnostic
- Malformed code → error message, exit 1, no file modification

## What is NOT in scope (v1)
- Operator spacing normalization (requires token-level analysis without desugaring)
- Import sorting / grouping
- Indentation level normalization (4-space to 2-space conversion)
- AST-based pretty printing (lossy due to parser desugaring)

## Success Criteria
- `synoema fmt file.sno` normalizes whitespace in-place
- `synoema fmt --check file.sno` exits 0 if already formatted, 1 otherwise
- `synoema fmt dir/` formats all `.sno` files recursively
- Idempotent: `format(format(code)) == format(code)`
- All existing examples/ pass `--check` after formatting
- Comments and string content preserved exactly
- Malformed code → error, not crash
- All existing tests pass, 0 warnings
