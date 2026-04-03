# Spec: String Stdlib in JIT

## Capability

Port 6 string builtin functions from interpreter-only to JIT via runtime FFI.

## Functions

### str_slice(s: String, from: Int, to: Int) → String
Byte-based substring extraction. Clamps `from`/`to` to `[0, len]`. Returns new tagged string.

### str_find(s: String, sub: String, from: Int) → Int
Byte-based substring search starting at position `from`. Returns first match index or -1. Empty substring returns `from`.

### str_starts_with(s: String, prefix: String) → Bool
Returns 1 (true) if `s` starts with `prefix`, else 0 (false). Returns untagged int (0/1), same as other bool-returning builtins.

### str_trim(s: String) → String
Removes leading and trailing ASCII whitespace. Returns new tagged string.

### str_len(s: String) → Int
Returns byte length of string as untagged i64.

### json_escape(s: String) → String
Escapes: `\` → `\\`, `"` → `\"`, `\n` → `\\n`, `\r` → `\\r`, `\t` → `\\t`. Returns new tagged string.

## ABI

All functions use `extern "C"` calling convention. String arguments are tagged i64 pointers (STR_TAG=2). String returns are tagged i64 pointers allocated via arena. Int/Bool returns are untagged i64.

## Invariants

- JIT output must match interpreter output for all inputs
- No new dependencies
- Arena-allocated strings (no malloc leaks)
- Existing tests must continue to pass
