---
id: string-interpolation
type: spec
status: done
---

# Spec: String Interpolation

## Requirement

Add `"text ${expr} text"` syntax for embedding expressions in strings.

## Syntax

```sno
name = "world"
msg = "hello ${name}"           -- "hello world"
math = "2+3 = ${show (2 + 3)}"  -- "2+3 = 5"
nested = "a ${show (length [1 2 3])} b"  -- "a 3 b"
```

## BPE Verification

`${` and `}` in string context — no new operators needed. Inside string literal, standard tokenization.

## Desugaring

```sno
"hello ${name}, you have ${show n} items"
-- desugars to:
"hello " ++ show name ++ ", you have " ++ show n ++ " items"
```

Rules:
- `${expr}` → `show expr` if expr type ≠ String
- `${expr}` → `expr` if expr type = String
- Empty segments omitted: `"${x}"` → `show x` (no empty string concat)

## Pipeline

1. **Lexer**: on `"`, scan for `${`. If found → emit `StringInterp` token sequence:
   `StringPart("hello ") Interp(tokens...) StringPart(", you have ") Interp(tokens...) StringPart(" items")`
2. **Parser**: `StringInterp` → `ExprKind::StringInterp(Vec<StringPart>)` where `StringPart = Lit(String) | Expr(Expr)`
3. **Desugar**: flatten to `Concat` chain: `"a" ++ show(e1) ++ "b" ++ show(e2) ++ "c"`
4. **Type checker**: each interpolated expr must be `Show`-able
5. **Interpreter/JIT**: no changes (handled by desugar)

## Escape

- `\${` → literal `${` (not interpolation)
- `\\` → literal `\`

## Acceptance Criteria

- [ ] `"hello ${name}"` parses and evaluates
- [ ] Nested expressions: `"${show (1 + 2)}"` works
- [ ] Empty interpolation: `"${"hello"}"` → `"hello"`
- [ ] Escape: `"\${x}"` → literal `${x}`
- [ ] Works in interpreter and JIT
- [ ] GBNF grammar updated
- [ ] docs/llm/ updated
- [ ] ≥8 tests
