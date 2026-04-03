# Design: String Stdlib in JIT (v2)

## Pattern

All 6 functions follow the existing FFI pattern (same as synoema_str_concat etc.):

```
runtime.rs:  pub extern "C" fn synoema_str_xxx(...) -> i64
compiler.rs: builder.symbol("synoema_str_xxx", runtime::synoema_str_xxx as *const u8)
compiler.rs: decl(self, "synoema_str_xxx", "str_xxx", &sigN)?;
```

Generic `funcs.get(&name)` dispatch in compile_expr handles calls automatically.

## Signature Mapping

| Function | Sig | Alias (funcs key) |
|----------|-----|-------------------|
| str_slice | sig3_ret | str_slice |
| str_find | sig3_ret | str_find |
| str_starts_with | sig2 | str_starts_with |
| str_trim | sig1 | str_trim |
| str_len | sig1 | str_len |
| json_escape | sig1 | json_escape |

## Runtime Pattern

Each function: extract raw ptr via `str_ptr(tagged)` → read len → get data slice → operate → return tagged string or plain i64.

## Files Modified

1. `lang/crates/synoema-codegen/src/runtime.rs` — 6 new functions
2. `lang/crates/synoema-codegen/src/compiler.rs` — 6 symbols + 6 declarations
3. `lang/crates/synoema-codegen/src/lib.rs` — 6+ tests
