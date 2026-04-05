# Spec: MCP Project Introspection Tools

## project_overview

**WHEN** LLM вызывает tool `project_overview` без параметров
**THEN** возвращается JSON:
```json
{
  "crates": [
    {"name": "synoema-lexer", "purpose": "Tokenization", "loc": 735, "tests": 82},
    ...
  ],
  "total_loc": 12000,
  "total_tests": 937,
  "warnings": 0,
  "dependencies_graph": {"synoema-parser": ["synoema-lexer"], ...}
}
```
- Ответ ≤300 токенов
- LOC считается по `.rs` файлам в `src/`
- Test count — по `#[test]` атрибутам в `src/` и `tests/`
- Dependencies — из `Cargo.toml` каждого crate (только internal deps)

## crate_info

**WHEN** LLM вызывает tool `crate_info` с параметром `crate: "synoema-types"`
**THEN** возвращается pub API surface:
```json
{
  "name": "synoema-types",
  "purpose": "Hindley-Milner type inference",
  "pub_functions": [
    {"name": "typecheck", "sig": "(&str) -> Result<TypeEnv, TypeError>", "file": "infer.rs", "line": 42}
  ],
  "pub_types": [
    {"name": "Type", "kind": "enum", "variants": ["TInt", "TBool", ...], "file": "types.rs", "line": 10}
  ],
  "pub_structs": [...],
  "internal_deps": ["synoema-lexer", "synoema-parser"],
  "loc": 1908,
  "tests": 61
}
```
- Ответ ≤500 токенов
- Парсинг через `syn`: извлечение `pub fn`, `pub struct`, `pub enum` с сигнатурами
- Если crate не найден — ошибка `unknown crate: <name>`

## file_summary

**WHEN** LLM вызывает tool `file_summary` с параметром `file: "lang/crates/synoema-eval/src/eval.rs"`
**THEN** возвращается список функций с сигнатурами (без тел):
```json
{
  "file": "eval.rs",
  "functions": [
    {"name": "eval_expr", "vis": "pub", "sig": "(env: &Env, expr: &Expr) -> Result<Value, Diagnostic>", "line": 45},
    {"name": "eval_binop", "vis": "priv", "sig": "(op: BinOp, l: &Value, r: &Value) -> Result<Value, String>", "line": 120}
  ],
  "structs": [...],
  "enums": [...],
  "impls": [...]
}
```
- Ответ ≤300 токенов
- Путь относительно repo root
- Если файл не найден — ошибка с did_you_mean (fuzzy match)

## search_code

**WHEN** LLM вызывает tool `search_code` с параметрами `query: "tagged pointer"` и опциональным `scope: "code"`
**THEN** возвращаются top-5 результатов:
```json
{
  "results": [
    {"file": "runtime.rs", "line": 45, "context": "// Tagged pointer ABI: bit 0 = list ..."},
    {"file": "compiler.rs", "line": 200, "context": "fn tag_value(val: i64, tag: u8) -> i64 ..."}
  ],
  "total_matches": 12
}
```
- Scope: `code` (только `.rs`), `docs` (только `.md`), `tests` (только `#[test]` блоки), `all` (default)
- Контекст: строка совпадения ± 1 строка
- Ответ ≤400 токенов
- Case-insensitive substring match

## get_context_for_edit

**WHEN** LLM вызывает tool `get_context_for_edit` с параметрами `file: "eval.rs"` и `line: 342`
**THEN** возвращается сфокусированный контекст:
```json
{
  "function": {"name": "eval_expr", "start": 300, "end": 420, "sig": "..."},
  "code": "... 20 строк вокруг line 342 ...",
  "local_vars": ["env: &Env", "expr: &Expr"],
  "imports_used": ["Value", "Diagnostic", "Expr"]
}
```
- Ответ ≤500 токенов
- Показывает функцию, содержащую строку
- Если строка вне функции — показывает ±20 строк
- Локальные переменные определяются через syn (let-binding analysis)
