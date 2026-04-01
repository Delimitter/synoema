# Hindley-Milner for LLMs: Type Inference Without Annotations

## Polymorphic Typing: Fewer Tokens, Stronger Guarantees

---

> **Who this is for.** If you've wondered whether it's possible to have strict typing without verbose annotations like Java or TypeScript — the answer is yes. This article explains how, and why it's critical for LLMs.

---

33.6% of all LLM-generated code failures are type errors. Can we eliminate them without making the LLM generate type annotations?

## The Problem: Types Cost Tokens

```typescript
// TypeScript: ~50% of tokens are type annotations
function add(a: number, b: number): number {
    return a + b;
}
// ~15 tokens, ~7 are type annotations
```

Each annotation means more tokens to generate, more context consumed (quadratic attention cost), and more opportunities for the LLM to make mistakes.

The ideal: **100% type safety with zero type annotations.**

## Hindley-Milner: A 1960s Solution to a 2020s Problem

The Hindley-Milner algorithm[^hm] is a type inference system developed by mathematicians Hindley (1969) and Milner (1978). It lets the compiler **automatically** determine the type of every expression, **requiring zero annotations**.

[^hm]: **Hindley-Milner (HM)** — an algorithm that determines the most general type of every expression from source code alone, with no type annotations. Mathematically proven to always find the **principal type** (most general type) — or report an error. Used in Haskell, OCaml, F#, Elm, Standard ML.

```
-- Synoema: zero annotations, 100% type safety
add x y = x + y
-- Compiler infers: Int → Int → Int

id x = x
-- Compiler infers: ∀a. a → a (polymorphic[^poly] type)

map f [] = []
map f (x:xs) = f x : map f xs
-- Compiler infers: ∀a b. (a → b) → List a → List b
```

[^poly]: **Polymorphic type** — a type that works with different data types. The function `id x = x` accepts a value of any type and returns it unchanged. Written `∀a. a → a`: "for any type `a`, accepts `a` and returns `a`." Like generics in Java/TypeScript, but without angle brackets.

The LLM doesn't need to generate a single type token. The compiler knows the types of all expressions.

## How It Works

Algorithm W[^algw] works in three steps:

[^algw]: **Algorithm W** — the specific implementation of Hindley-Milner inference, proposed by Damas and Milner in 1982. Traverses the program's AST bottom-up, collecting type constraints and solving them via unification.

**Step 1: Constraint generation.** For each expression, the compiler creates a type variable[^typevar] and records constraints. For `x + y`: `x : τ₁`, `y : τ₂`, `(+) : Int → Int → Int`, constraints: `τ₁ = Int`, `τ₂ = Int`.

[^typevar]: **Type variable** — a placeholder for an unknown type. Denoted by Greek letters (τ, α, β) or Latin (a, b). During inference, placeholders are replaced with concrete types through unification.

**Step 2: Unification[^unify].** The constraint system is solved: type variables are replaced with concrete types. If constraints conflict — the compiler reports an error.

[^unify]: **Unification** — the process of finding a substitution that makes two type expressions equal. For example, unifying `List a` and `List Int` yields `a = Int`. Unifying `Int` and `String` is an error.

**Step 3: Generalization[^gen].** Remaining free variables are generalized: `id : a → a` becomes `id : ∀a. a → a` — the function works with any type.

[^gen]: **Generalization** — turning a concrete type into a polymorphic one. If after inference variable `a` remains free, it's generalized: `a → a` becomes `∀a. a → a`. This allows using `id` as both `id 42` (Int) and `id true` (Bool) in the same program.

Key property: **HM always finds the most general type.** No guessing or specifying needed.

## Let-Polymorphism

Synoema implements let-polymorphism[^letpoly]:

[^letpoly]: **Let-polymorphism** — a mechanism where a variable defined via `let` (or `=` in Synoema) gets a polymorphic type. This means `id` can simultaneously be `Int → Int` and `Bool → Bool` in one program. Without let-polymorphism, `id` would be locked to one type.

```
id x = x
main =
  a = id 42        -- id used as Int → Int
  b = id true       -- id used as Bool → Bool
  a                 -- No error! id is polymorphic.
```

In Go (before 1.18): impossible without `interface{}`. In Python: "works" via duck typing — but without guarantees.

## Interaction with Constrained Decoding

At each code generation step, the compiler can determine **valid types** for the next expression. This creates a second constraint layer on top of grammar:

```
-- LLM generates: map ??? [1 2 3]
-- Compiler knows: ??? : Int → τ  (function from Int)
-- Valid: \x -> x + 1, \x -> x * 2, show
-- Invalid: \x -> x ++ "hello" (String ≠ Int)
```

Grammar constraint: "an expression goes here."
Type constraint adds: "and it must have type `Int → τ`."

Together, they narrow the space of valid continuations by orders of magnitude.

## Comparison

| Approach | Type guarantees | Tokens on types | Runtime errors |
|----------|----------------|-----------------|---------------|
| Python (duck typing) | None | 0 | Many |
| TypeScript | Yes | ~30–50% of code | Few |
| Java | Yes | ~40–60% of code | Few |
| Synoema (HM) | **Yes** | **0** | **None** |

Synoema: **maximum guarantees, zero cost.**

## What's Next

Next: **Synoema launch article** — all three levers together, 264 tests, 10 examples, GBNF grammar, and getting started instructions.

---

*Fifth article in "Token Economics of Code." HM type inference implemented in 1,453 lines of Rust with 42 tests.*

---

## Glossary

| Term | Explanation |
|------|-----------|
| **Hindley-Milner** | Automatic type inference algorithm. Determines types without annotations |
| **Algorithm W** | HM implementation. Traverses AST bottom-up, collects constraints |
| **Type variable** | Placeholder for an unknown type, resolved through unification |
| **Unification** | Process of equating two type expressions. Foundation of type inference |
| **Generalization** | Converting concrete type to polymorphic: `a → a` → `∀a. a → a` |
| **Let-polymorphism** | One function can have different types in different contexts |
| **Polymorphic type** | Type working with any data type. `∀a. a → a` = "for any type" |
| **Principal type** | Most general type of an expression. HM always finds it |
| **Duck typing** | Python's principle: types unchecked until runtime |
