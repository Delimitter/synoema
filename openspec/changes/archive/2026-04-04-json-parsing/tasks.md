# JSON Parsing -- Tasks

- [x] Define JsonValue ADT in prelude.sno
- [x] Implement json_get as pure prelude function
- [x] Implement json_parse recursive descent parser in eval.rs (interpreter)
- [x] Register json_parse as builtin in eval.rs builtin_env
- [x] Implement json_parse as extern "C" in runtime.rs (JIT)
- [x] Register json_parse symbol in compiler.rs builder
- [x] Register json_parse function declaration in compiler.rs
- [x] Register json_parse type (String -> Result JsonValue String) in infer.rs
- [x] Add tests: json_parse_null, json_parse_true, json_parse_false
- [x] Add tests: json_parse_number, json_parse_negative_number
- [x] Add tests: json_parse_string, json_parse_string_escape
- [x] Add tests: json_parse_empty_array, json_parse_array
- [x] Add tests: json_parse_empty_object, json_parse_object
- [x] Add tests: json_parse_nested
- [x] Add tests: json_parse_error_trailing, json_parse_error_empty
- [x] Add tests: json_parse_with_unwrap, json_parse_json_get
- [x] Add test: json_parse_whitespace
- [x] Verify cargo test passes with 0 failures, 0 warnings
