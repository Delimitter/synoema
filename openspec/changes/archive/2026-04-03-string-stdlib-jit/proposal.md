# Proposal: String Stdlib in JIT

## Problem

Phase 18 added 6 string builtins to the interpreter (`str_slice`, `str_find`, `str_starts_with`, `str_trim`, `str_len`, `json_escape`), but they are not available in JIT mode. Programs like `stress_server.sno` that use these functions can only run via `synoema run`, losing the 4.4x JIT performance advantage.

## Solution

Port all 6 string builtins to JIT as runtime FFI functions. The existing infrastructure (tagged pointer ABI for strings, arena allocator, `funcs.get` dispatch) makes this straightforward:

1. Implement `extern "C"` functions in `runtime.rs` using the existing `str_ptr()` / `StrNode` / `synoema_str_new()` patterns
2. Register symbols in `Compiler::new()`
3. Declare function signatures in `declare_runtime_functions()`
4. The generic `funcs.get(&name)` dispatch at compile_expr handles the rest automatically

## Functions to Port

| Function | Signature | Cranelift sig |
|----------|-----------|---------------|
| `str_slice` | `(Str, Int, Int) → Str` | sig3_ret |
| `str_find` | `(Str, Str, Int) → Int` | sig3_ret |
| `str_starts_with` | `(Str, Str) → Bool` | sig2 |
| `str_trim` | `(Str) → Str` | sig1 |
| `str_len` | `(Str) → Int` | sig1 |
| `json_escape` | `(Str) → Str` | sig1 |

## Scope

- 6 runtime FFI functions
- Symbol registration + declaration (boilerplate)
- JIT codegen tests (1 per function minimum)
- No changes to lexer, parser, type checker, or Core IR needed
- No new dependencies

## Out of Scope

- Network/IO builtins (`tcp_listen`, `file_read`, etc.) — separate change
- TCO in JIT — separate change
