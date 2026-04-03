---
id: spec-doctests
type: spec
status: done
---

# Spec: Doctests (--- example: проверяются при тестировании)

## Формат

Строка doc-comment, начинающаяся с `example:`, является doctest:

```
--- example: <expr>                    → eval, print result (smoke test)
--- example: <expr> == <expected>      → eval both, assert equality
```

### Парсинг doctest-строки

1. Из `Vec<String>` doc-комментариев выделить строки с prefix `example:`
2. Trim prefix → получить выражение: `qsort [3 1] == [1 3]`
3. Если содержит `==` на top-level (не внутри скобок):
   - Split по `==` → lhs, rhs
   - Parse обе стороны как Expr
   - Eval обе → assert equality
4. Если не содержит `==`:
   - Parse как Expr
   - Eval → assert no runtime error (smoke test)

### Контекст исполнения

Doctest выполняется в контексте **всего модуля**. Все функции модуля доступны:

```synoema
--- example: fac 5 == 120
fac 0 = 1
fac n = n * fac (n - 1)
```

Doctest `fac 5 == 120` может вызвать `fac` потому что весь модуль загружен.

**Порядок:** модуль полностью загружен → doctests запускаются после.

### Множественные examples

```synoema
--- Sort a list.
--- example: qsort [3 1 2] == [1 2 3]
--- example: qsort [] == []
--- example: qsort [1] == [1]
qsort [] = []
qsort (p:xs) = ...
```

Все три example проверяются.

## CLI интерфейс

```bash
# Run all doctests in a file
synoema test examples/quicksort.sno

# Run all doctests in all .sno files in directory
synoema test examples/

# Run doctests + regular cargo tests
# (integration: synoema-repl/tests/ вызывает doctest runner)
```

### Вывод

Успех:
```
  doctests: quicksort.sno — 3/3 passed
  doctests: geometry.sno — 2/2 passed
  Total: 5/5 doctests passed
```

Неудача:
```
  FAIL: quicksort.sno:2 — example: qsort [3 1 2] == [1 2 3]
    Left:  [1, 2, 3]
    Right: [3, 1, 2]
  doctests: quicksort.sno — 2/3 passed (1 FAILED)
```

## Архитектура исполнения

```
synoema test file.sno
    │
    ├── 1. Lex + Parse → AST (с doc: Vec<String>)
    │
    ├── 2. Extract doctests:
    │      для каждого Decl с doc:
    │        filter lines starting with "example:"
    │        → Vec<(Span, String)>  (location + expression)
    │
    ├── 3. Type-check модуль (как обычно)
    │
    ├── 4. Load module в eval/JIT environment
    │
    └── 5. Для каждого doctest:
           parse expression → type-check → eval
           if has == → compare lhs/rhs
           if no == → assert no panic
           report pass/fail
```

### Isolation

Каждый doctest:
- Имеет доступ ко всем определениям модуля (read-only)
- НЕ мутирует environment (Synoema immutable by design)
- НЕ может повлиять на другие doctests (no shared state)

Проблема Jupyter (hidden state, 24% reproducibility) невозможна:
- Нет мутации → нет stale state
- Нет cell ordering dependency → каждый doctest independent

## Перформанс

- Doctests запускаются ТОЛЬКО при `synoema test`, не при `synoema run` или `synoema jit`
- Overhead = parse + typecheck + eval для каждого example-выражения
- Оценка: ~20-50ms на doctest (comparable с unit test)
- 100 doctests ≈ 2-5 секунд

## Влияние на токенную экономику

LLM генерирует doctests **только если запрошена документация**:
```
Prompt: "напиши quicksort"        → код без --- (0 overhead)
Prompt: "напиши quicksort с docs" → код с --- example: (~10 extra tokens)
```

При чтении: `--- example: qsort [3 1] == [1 3]` = ~12 BPE tokens.
Это дешевле чем LLM потратит на вывод поведения из кода (~50+ tokens reasoning).
