---
id: proposal
type: proposal
status: draft
---

# Proposal: derive(Show, Eq, Ord) для ADT

## Problem Statement

Synoema поддерживает ADT и trait/impl, но реализация type class методов — полностью ручная:

```sno
-- Сейчас: 6 строк ручного кода для 3 конструктора
Color = Red | Green | Blue

impl Show Color
  show Red = "Red"
  show Green = "Green"
  show Blue = "Blue"
```

Для каждого нового ADT нужно писать boilerplate для Show, Eq, Ord. LLM тоже вынуждена генерировать эти уравнения вручную — лишние токены, больше вероятность ошибки.

С derive:
```sno
Color = Red | Green | Blue
  deriving (Show, Eq, Ord)
```

1 строка вместо 6. Для ADT с полями экономия ещё больше:
```sno
Maybe a = Just a | None
  deriving (Show, Eq)

-- Генерируется автоматически:
-- show (Just x) = "Just " ++ show x
-- show None = "None"
-- eq (Just x) (Just y) = eq x y
-- eq None None = true
-- eq _ _ = false
```

## BPE Token Impact

```
Ручной impl Show Color (3 constructors):  ~18 tokens
derive (Show, Eq, Ord):                   ~8 tokens
Экономия:                                 ~56%
```

Для ADT с 5+ конструкторами экономия >70%.

## Approach: Compile-Time Synthesis

Стратегия: **генерация ImplDecl на этапе type checking**. Синтезированные impl неотличимы от ручных → zero changes в eval и codegen.

```
TypeDef с derives
      │
      ▼ synthesize_derived_impls()
      │
      ├─ derive_show() → ImplDecl (show equations)
      ├─ derive_eq()   → ImplDecl (eq equations)
      └─ derive_ord()  → ImplDecl (cmp equations by variant order)
      │
      ▼ Existing pipeline (без изменений)

      eval: prepend equations → dispatch
      JIT:  compile equations → native code
```

## Scope

- Parser: +45 LOC (parse `deriving (X, Y)`)
- AST: +5 LOC (derives field)
- Type Checker: +250 LOC (synthesize 3 traits)
- Tests: +100 LOC (~15 тестов)
- Docs + GBNF: +30 LOC
- **Eval/Codegen: 0 LOC** — ключевое свойство дизайна

## Success Criteria

- [ ] `deriving (Show)` автоматически генерирует show для ADT без полей
- [ ] `deriving (Show)` автоматически генерирует рекурсивный show для ADT с полями
- [ ] `deriving (Eq)` генерирует structural equality
- [ ] `deriving (Ord)` генерирует comparison по порядку вариантов
- [ ] Работает и в interpreter, и в JIT
- [ ] Ручной `impl Show` перекрывает derive (приоритет ручного)
- [ ] GBNF grammar обновлена
- [ ] BPE verified: `deriving` = 1 token
- [ ] ≥15 тестов, `cargo test` clean, 0 warnings
- [ ] `docs/llm/synoema.md` обновлена

## Non-Goals

- Полная система type classes (constraints, dictionary-passing, superclasses)
- derive для пользовательских traits
- derive для Records (только ADT)
- Functor, Monad, и прочие HKT-based classes
