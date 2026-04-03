---
id: type-alias
type: spec
status: done
---

# Spec: Type Aliases

## Requirement

Add `type Name params = TypeExpr` syntax for type abbreviations.

## Syntax

```sno
type Pos = {x : Int, y : Int}
type Pair a b = {fst : a, snd : b}
type IntList = [Int]
```

## BPE Verification

`type` = 1 BPE token (cl100k_base) ✓

## Semantics

- Pure textual substitution (no new type, no nominal distinction)
- `type Pos = {x : Int, y : Int}` → everywhere `Pos` appears in type signatures, replace with `{x : Int, y : Int}`
- Parametric: `type Pair a b = ...` → substitutes type arguments
- No recursive type aliases (error if detected)
- Scoped within module (visible after `use`)

## AST Change

New `Decl` variant:
```
Decl::TypeAlias {
    name: String,
    params: Vec<String>,
    body: TypeExpr,
    span: Span,
}
```

## Pipeline

1. **Parser**: parse `type Name params = TypeExpr` as new Decl
2. **Type checker**: before inference, expand all aliases in type expressions
3. **Desugar**: no changes (aliases don't appear in Core IR)
4. **Interpreter/JIT**: no changes (purely type-level)

## Acceptance Criteria

- [ ] `type Pos = {x : Int, y : Int}` parses
- [ ] `f : Pos -> Int` resolves to `f : {x : Int, y : Int} -> Int`
- [ ] Parametric aliases work: `type Pair a b = {fst : a, snd : b}`
- [ ] Recursive alias detected and reported as error
- [ ] Works inside modules: `mod M` → `type T = ...` → `use M (T)`
- [ ] GBNF grammar updated
- [ ] docs/llm/ updated
- [ ] ≥5 tests
