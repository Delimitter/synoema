# Spec: Linear Types (Phase A)

## Capability: linear-arrow

### Синтаксис

Линейная стрелка записывается как `-o` (один BPE-токен cl100k_base).

```
f : Int -o Int       -- функция, потребляющая аргумент ровно 1 раз
g : Int -> Int       -- обычная функция (unrestricted, как сейчас)
```

### Семантика

**Правило линейности (два правила — аналог Austral):**

1. **Use-Once**: переменная с линейным типом должна быть использована ровно 1 раз
2. **No-Aliasing**: линейную переменную нельзя передать в два места одновременно

```
-- ОШИБКА: использована дважды
bad x = (x, x)    -- если x : A -o B, то x нельзя дублировать

-- ОШИБКА: не использована
drop x = 42       -- если x : A -o B, то x нельзя выбросить

-- OK: использована ровно раз
good x = x + 1
```

### Opt-in совместимость

- Существующие программы (без `-o`) не изменяются
- Unrestricted переменные (`->`) можно использовать сколько угодно раз
- Линейную переменную можно передать в unrestricted-функцию (weakening запрещён в строгой линейности, но для opt-in совместимости — разрешить как warning, не error)

### BPE-alignment

| Токен | BPE (cl100k_base) | Статус |
|-------|-------------------|--------|
| `-o`  | 1 токен           | ✓ |
| `linear` | 1 токен      | резерв для будущего |

### Типовые правила

```
Γ, x :₁ A ⊢ e : B
────────────────────  (Lam-Linear)
Γ ⊢ λx. e : A -o B

Γ₁ ⊢ f : A -o B    Γ₂ ⊢ a : A    (Γ₁, Γ₂ disjoint)
─────────────────────────────────  (App-Linear)
       Γ₁, Γ₂ ⊢ f a : B
```

### Ограничения Phase A

- Только на стрелках (`-o`), не на переменных напрямую
- Нет rank-2 линейности (линейные аргументы к полиморфным функциям)
- Нет линейных record полей
- Case expressions: каждая ветвь должна использовать linear переменные одинаково

### Примеры ошибок

```
error[linear_unused] at 1:8: linear variable 'x' must be used exactly once
  1 | drop x = 42
              ^^^
  = note: 'x' has linear type and was never consumed

error[linear_duplicate] at 1:10: linear variable 'x' used more than once
  1 | copy x = (x, x)
                   ^
  = note: second use of linear variable 'x'
  = note: linear values cannot be duplicated
```

## Capability: linear-type-checking

### Usage Tracking

Type checker отслеживает usage count для каждой переменной в scope:

```
type Usage = Unused | UsedOnce | UsedMore
```

При выходе из scope, для каждой linear переменной:
- `Unused` → `error[linear_unused]`
- `UsedOnce` → OK
- `UsedMore` → `error[linear_duplicate]`

### Case branches

Все ветви case должны использовать linear переменные из enclosing scope одинаково:

```
-- OK: обе ветви используют x ровно раз
f x cond = ? cond -> g x : h x

-- ОШИБКА: одна ветвь использует, другая нет
bad x cond = ? cond -> g x : 42
```

### Interaction с let-polymorphism

Линейные переменные НЕ обобщаются (не могут быть полиморфными):

```
-- ОШИБКА: нельзя использовать id как линейную переменную дважды
bad = let id x = x in (id 1, id true)  -- id тут unrestricted, OK
-- но:
bad2 = let f -o = ... in ...  -- если f сам линейный, нельзя использовать дважды
```
