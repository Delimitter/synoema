# Spec: MCP Dynamic Recipes

## recipe tool

**WHEN** LLM вызывает tool `recipe` с параметром `task: "add_operator"`
**THEN** сервер анализирует текущий AST кодовой базы через `syn` и возвращает актуальный пошаговый рецепт:
```json
{
  "task": "add_operator",
  "steps": [
    {
      "step": 1,
      "file": "lang/crates/synoema-lexer/src/token.rs",
      "action": "Add variant to enum Token",
      "location": {"after_line": 82, "pattern": "OpCompose"},
      "template": "OpMyOp,  // \"myop\" — description",
      "existing_pattern": "OpPipe,   // \"|>\"\nOpCompose, // \">>\""
    },
    {
      "step": 2,
      "file": "lang/crates/synoema-lexer/src/scanner.rs",
      "action": "Add scan rule to match block",
      "location": {"after_line": 140, "pattern": "\">>\" => Token::OpCompose"},
      "template": "\"myop\" => Token::OpMyOp,"
    },
    {
      "step": 3,
      "file": "lang/crates/synoema-parser/src/parser.rs",
      "action": "Add precedence level",
      "location": {"in_function": "precedence", "line": 85},
      "template": "Token::OpMyOp => 6,  // same as similar ops"
    }
  ],
  "verify": ["cargo test -p synoema-lexer", "cargo test -p synoema-parser"],
  "warnings": ["BPE: verify new operator is 1 token via tools/bpe-verify/"]
}
```

## Поддерживаемые рецепты

### add_operator
- Парсит: `token.rs` (enum Token), `scanner.rs` (match block), `parser.rs` (precedence fn)
- Шаги: token variant → scan rule → precedence → tests

### add_builtin
- Парсит: `eval.rs` (builtin dispatch), `codegen/compiler.rs` (JIT builtins), `codegen/runtime.rs` (FFI)
- Шаги: interpreter builtin → JIT FFI function → JIT registration → tests

### add_type
- Парсит: `types.rs` (Type enum), `infer.rs` (unification rules), `core_ir.rs` (desugar)
- Шаги: type variant → inference rule → desugar support → eval → codegen → tests

### fix_from_error
- Входные параметры: `error_code: "type_mismatch"`, `file: "eval.rs"`, `line: 342`
- Парсит: целевой файл, ищет контекст ошибки
- Шаги: locate error → understand context → suggest fix pattern

## Динамичность

- Все номера строк определяются в момент вызова через `syn` парсинг
- `existing_pattern` показывает реальный код в точке вставки
- Если структура кода изменилась (enum переименован, функция перемещена) — рецепт адаптируется
- Если структура не обнаружена — возвращается ошибка с описанием что именно не найдено

## Ответы

- Каждый рецепт ≤500 токенов
- Формат: JSON со steps (массив), verify (команды проверки), warnings (предупреждения)
- Если задача не поддерживается — `{"error": "unknown recipe", "available": ["add_operator", "add_builtin", "add_type", "fix_from_error"]}`
