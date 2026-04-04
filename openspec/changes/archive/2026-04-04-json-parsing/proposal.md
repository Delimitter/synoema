# JSON Parsing

## Summary

Add a built-in JSON parser to Synoema, enabling programs to parse JSON strings into a structured `JsonValue` ADT and access fields via `json_get`.

## Motivation

LLM-generated code frequently works with JSON data (API responses, configuration files, structured output). A built-in JSON parser allows Synoema programs to consume JSON without external dependencies, completing the JSON workflow alongside the existing `json_escape` builtin.

## Design

### JsonValue ADT (prelude)

```
JsonValue = JNull | JBool Bool | JNum Int | JStr String | JArr (List JsonValue) | JObj (List (Pair String JsonValue))
```

Objects use `List (Pair String JsonValue)` to reuse the existing `Pair` and `Map` infrastructure from the prelude.

### Builtins

- `json_parse : String -> Result JsonValue String` -- recursive descent parser implemented as `extern "C"` runtime function (interpreter + JIT)
- `json_get : String -> JsonValue -> Result JsonValue String` -- pure prelude function using pattern matching on `JObj` + `map_lookup_list`

### Parser features

- Supports: null, true, false, integers, strings (with escape sequences), arrays, objects
- Returns `Ok JsonValue` on success, `Err String` with position-based error message on failure
- Validates no trailing content after top-level value

## Files changed

- `lang/prelude/prelude.sno` -- JsonValue ADT + json_get
- `lang/crates/synoema-eval/src/eval.rs` -- interpreter builtin registration + recursive descent parser
- `lang/crates/synoema-codegen/src/runtime.rs` -- JIT runtime extern C json_parse + recursive descent parser
- `lang/crates/synoema-codegen/src/compiler.rs` -- JIT symbol + function declaration registration
- `lang/crates/synoema-types/src/infer.rs` -- type signature for json_parse
- `lang/crates/synoema-eval/src/tests.rs` -- 17 tests covering all JSON types, nesting, errors, integration with prelude
