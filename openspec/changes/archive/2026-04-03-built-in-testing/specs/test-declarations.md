# Spec: test declarations

## Syntax

```
test <string-literal> = <expr>
```

`test` — keyword (1 BPE-токен). Тело — произвольное выражение, приводимое к Bool.

## Семантика

- `test` — top-level declaration, не экспортируется, не участвует в type inference других деклараций
- Тело вычисляется в контексте всех деклараций файла (может вызывать любые функции)
- Результат: `true` → pass, `false` → fail, runtime error → fail с диагностикой
- Тесты выполняются в порядке объявления

## Примеры

```
test "fact base case" = fact 0 == 1
test "fact recursive" = fact 5 == 120
test "sort then take" = take 2 (qsort [5 3 1 4 2]) == [1 2]
test "empty sort" = qsort [] == []
```

## AST

Новый вариант `Decl::Test { name: String, body: Expr, span: Span }`.

## Runner

`synoema test <path>` запускает:
1. Doctests (как сейчас)
2. Test-декларации (новое)

Опция `--filter <substring>` — запускает только тесты, чьё имя содержит подстроку.
