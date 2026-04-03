# Design: String Interpolation

## Approach: Lexer-level tokenization + Parser-level desugaring

### Option A (chosen): Lexer splits into fragments + parser assembles

The lexer breaks `"hello ${x} world"` into a token sequence:

```
StringFragment("hello ")  InterpStart  LowerId("x")  InterpEnd  StringFragment(" world")
```

The parser reassembles: if the first token after `"` is `StringFragment` (or `InterpStart`), it parses an interpolated string expression.

**Pros:** Clean separation, parser can parse arbitrary expressions inside `${}`, nesting handled naturally by recursive lexing.

**Cons:** New token types needed.

### Option B (rejected): Parser-only with re-lexing

Parser sees a regular `Str` token, re-parses its content looking for `${`. Would require the parser to have its own mini-lexer.

**Rejected:** Violates separation of concerns, harder to handle nesting.

## Lexer Changes

### New Tokens

```rust
Token::StringFragment(String)  // Text segment of interpolated string
Token::InterpStart             // ${ — starts interpolation
Token::InterpEnd               // } that closes interpolation (NOT regular })
```

### scan_string modification

When encountering `${` inside a string:
1. Emit `StringFragment` for accumulated text
2. Emit `InterpStart`
3. Return to normal lexing mode
4. Track brace depth to find matching `}`
5. On matching `}`, emit `InterpEnd`
6. Resume string scanning for remaining content

### State tracking

Add `interp_depth: Vec<usize>` to Scanner:
- On `InterpStart`: push brace_depth=0
- On `{` inside interpolation: increment top counter
- On `}` inside interpolation: if counter > 0 decrement, else pop + emit `InterpEnd`

### Escape handling

- `\$` → literal `$` character (no interpolation)
- `$` not followed by `{` → literal `$` character

## Parser Changes

### parse_interpolated_string

When parser encounters `StringFragment` or (`InterpStart` at string position):

```
parse_interp_string() → Expr:
  segments: Vec<Expr> = []
  loop:
    if StringFragment(s): segments.push(Lit(Str(s)))
    if InterpStart:
      expr = parse_expr()
      expect InterpEnd
      segments.push(App(Var("show"), expr))
    if next is not StringFragment/InterpStart: break

  fold segments with BinOp(Concat)
```

### Optimization

- If only one segment and it's a Lit(Str), return as-is (no interpolation)
- Elide empty string fragments
- Single `${expr}` with no surrounding text → `show expr`

## Testing Strategy

1. **Lexer tests:** Fragment tokenization, nesting, escapes
2. **Parser tests:** Desugaring to correct AST
3. **Eval tests:** End-to-end in interpreter
4. **JIT tests:** End-to-end in codegen stress tests
5. **Edge cases:** Empty interpolation, consecutive interpolations, nested strings

## GBNF Grammar Update

```ebnf
string = '"' string-content* '"'
string-content = string-char+ | interpolation
interpolation = '${' expr '}'
string-char = <any except '"', '\', '${'> | escape | '$' (!'{')
escape = '\n' | '\t' | '\\' | '\"' | '\$' | '\0'
```
