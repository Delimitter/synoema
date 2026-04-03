# Spec: String Stdlib in JIT (v2)

## Functions

| Function | Signature | Returns |
|----------|-----------|---------|
| str_slice(s, from, to) | 3×i64→i64 | tagged string |
| str_find(s, sub, from) | 3×i64→i64 | untagged int (-1 if not found) |
| str_starts_with(s, prefix) | 2×i64→i64 | untagged int (0/1) |
| str_trim(s) | 1×i64→i64 | tagged string |
| str_len(s) | 1×i64→i64 | untagged int |
| json_escape(s) | 1×i64→i64 | tagged string |

## ABI

All use `extern "C"`. String args are tagged i64 (STR_TAG=2). String returns allocated via arena. Int/Bool returns are plain i64.

## Edge Cases

- str_slice: clamp to [0,len], ensure to≥from
- str_find: empty needle→from, from>len→-1
- str_starts_with: empty prefix→true, prefix>string→false
- str_trim: ASCII whitespace only
- json_escape: escapes \, ", \n, \r, \t

## Invariants

- JIT output must match interpreter for all inputs
- No new dependencies
- Arena-allocated (no leaks)
- 634+ existing tests pass
