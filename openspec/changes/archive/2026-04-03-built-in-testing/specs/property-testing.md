# Spec: property-based testing

## Syntax

```
test "name" = prop <var> -> <Bool-expr>
test "name" = prop <var> <var> -> <Bool-expr>
```

`prop` — keyword (1 BPE-токен). Переменные типизируются через inference из тела.

## Conditional properties

```
test "name" = prop n -> n >= 0 implies fact n > 0
```

`implies` — keyword (1 BPE-токен). Семантика: `false implies _` = discard (не fail).

## Type-driven generation

Тип переменной определяет генератор:

| Тип | Генерация |
|-----|-----------|
| Int | Случайные из [-100, 100] |
| Bool | true, false |
| String | Случайные 0..8 ascii символов |
| [a] | Списки длины 0..10, элементы по типу `a` |

Генерация выполняется в interpreter. 100 случайных входов на property.

## Семантика

- `prop` создаёт замыкание `vars -> Bool`
- Runner генерирует 100 наборов входов, вызывает замыкание
- Все `true` → pass
- Любой `false` → fail, показать counterexample (конкретные значения)
- `implies`: если предусловие `false`, кейс отбрасывается (не считается ни pass, ни fail)
- Если >90% кейсов отброшены — warning "trivial property"

## Примеры

```
test "reverse involution" = prop xs -> reverse (reverse xs) == xs
test "sort idempotent" = prop xs -> qsort (qsort xs) == qsort xs
test "sort preserves length" = prop xs -> length (qsort xs) == length xs
test "fact positive" = prop n -> n >= 0 implies fact n > 0
```

## AST

Новые варианты:
- `ExprKind::Prop(Vec<String>, Box<Expr>)` — `prop x y -> body`
- `ExprKind::Implies(Box<Expr>, Box<Expr>)` — `cond implies body`
