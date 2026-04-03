# Proposal: String Stdlib in JIT (v2)

## Problem

6 string builtins (str_slice, str_find, str_starts_with, str_trim, str_len, json_escape) exist in interpreter but not in JIT runtime. Programs using these functions fail in `synoema jit` mode.

## Solution

Port all 6 functions to JIT via the existing FFI pattern: `extern "C"` functions in runtime.rs, symbol registration + declaration in compiler.rs. No new dependencies, no architecture changes.

## Scope

- 6 new `extern "C"` functions in runtime.rs
- 6 symbol registrations in compiler.rs
- 6 function declarations in compiler.rs
- 6+ JIT tests in lib.rs
- All existing 634 tests must continue to pass
