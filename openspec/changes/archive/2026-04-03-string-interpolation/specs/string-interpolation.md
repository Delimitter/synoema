# Spec: String Interpolation

## Syntax

```ebnf
string_literal = '"' string_content* '"'
string_content = escape_seq | interpolation | plain_char
interpolation  = '${' expr '}'
escape_seq     = '\n' | '\r' | '\t' | '\\' | '\"' | '\0' | '\$'
plain_char     = <any char except '"', '\', '${'>
```

## Semantics (desugaring)

String interpolation — syntactic sugar, resolved at parse time.

### Rules

1. `"text"` (no interpolation) → `Lit(Str("text"))` — unchanged
2. `"${expr}"` → `App(Var("show"), expr)`
3. `"text${expr}"` → `BinOp(Concat, Lit(Str("text")), App(Var("show"), expr))`
4. `"${e1}mid${e2}"` → `BinOp(Concat, BinOp(Concat, App(Var("show"), e1), Lit(Str("mid"))), App(Var("show"), e2))`
5. Empty segments are elided: `"${x}"` does NOT produce `"" ++ show x`

### Nesting

Interpolation expressions may contain nested string literals, including nested interpolations:
```sno
"outer ${inner ++ "nested ${x}"} end"
```

The lexer tracks brace depth to find the matching `}`.

### Escape

`\$` produces literal `$`. A `$` not followed by `{` is a literal `$`.

## BPE Alignment

| Symbol | cl100k_base tokens |
|--------|-------------------|
| `${`   | 1 (id: 2420)      |
| `}`    | 1 (id: 92)        |
| `\$`   | 1 (id: 59456)     |

## Affected Crates

| Crate | Change |
|-------|--------|
| synoema-lexer | New token type `InterpStart`/`InterpEnd`, string fragment scanning |
| synoema-parser | Desugar interpolated strings to `show` + `++` |
| synoema-types | No changes |
| synoema-core | No changes |
| synoema-eval | No changes |
| synoema-codegen | No changes |

## Non-Goals

- Format specifiers (`${x:.2f}`) — future work
- Raw strings (`r"..."`) — future work
- Multi-line string interpolation — already works (strings are single-line by design)
