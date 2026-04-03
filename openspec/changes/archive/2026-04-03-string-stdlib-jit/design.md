# Design: String Stdlib in JIT

## Architecture

All 6 functions follow the existing FFI pattern:

```
runtime.rs: pub extern "C" fn synoema_str_xxx(...) -> i64
compiler.rs: builder.symbol("synoema_str_xxx", runtime::synoema_str_xxx as *const u8)
compiler.rs: decl(self, "synoema_str_xxx", "str_xxx", &sigN)?;
```

The generic `funcs.get(&name)` dispatch in `compile_expr` (line ~709) handles call codegen automatically — no special cases needed in the compiler.

## Signature Mapping

| Function | Params | Existing sig | Alias (funcs key) |
|----------|--------|-------------|-------------------|
| str_slice | 3×i64→i64 | sig3_ret | str_slice |
| str_find | 3×i64→i64 | sig3_ret | str_find |
| str_starts_with | 2×i64→i64 | sig2 | str_starts_with |
| str_trim | 1×i64→i64 | sig1 | str_trim |
| str_len | 1×i64→i64 | sig1 | str_len |
| json_escape | 1×i64→i64 | sig1 | json_escape |

## Runtime Implementation Pattern

Each function:
1. Extracts raw pointer from tagged value via `str_ptr(tagged)`
2. Reads `len` from `StrNode.len` field
3. Gets data via `ptr.add(1) as *const u8` → `slice::from_raw_parts(data, len)`
4. Performs the operation
5. Returns result: strings via `synoema_str_new()`, ints as plain i64

## Edge Cases

- `str_slice`: clamp both bounds to `[0, len]`, ensure `to >= from`
- `str_find`: empty substring → return `from`; `from > len` → return -1
- `str_starts_with`: empty prefix → true; prefix longer than string → false
- `str_trim`: ASCII whitespace only (matches interpreter behavior)
- `json_escape`: byte-by-byte scanning, output buffer may be up to 2x input

## Testing Strategy

One test per function in `synoema-codegen/src/lib.rs` using `jit()` helper. Each test verifies JIT output matches expected value. Test names: `jit_str_slice`, `jit_str_find`, etc.

## Files Modified

1. `lang/crates/synoema-codegen/src/runtime.rs` — 6 new `extern "C"` functions
2. `lang/crates/synoema-codegen/src/compiler.rs` — 6 symbol registrations + 6 declarations
3. `lang/crates/synoema-codegen/src/lib.rs` — 6+ new tests
