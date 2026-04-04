# Spec: Result Type + Error Handling

## Текущее состояние: РЕАЛИЗОВАНО

Всё ниже **уже работает** в текущей кодовой базе.

### Prelude (`lang/prelude/prelude.sno`)

```sno
Result a e = Ok a | Err e

map_ok f (Ok x) = Ok (f x)
map_ok f (Err e) = Err e

map_err f (Ok x) = Ok x
map_err f (Err e) = Err (f e)

unwrap (Ok x) = x
unwrap (Err e) = error ("unwrap called on Err: " ++ show e)

unwrap_or def (Ok x) = x
unwrap_or def (Err _) = def

is_ok (Ok _) = true
is_ok (Err _) = false

is_err (Ok _) = false
is_err (Err _) = true

and_then f (Ok x) = f x
and_then f (Err e) = Err e
```

### error function

`error : ∀a. String -> a` — builtin в:
- Type checker: `infer.rs:480-485` — полиморфный return type (bottom)
- Eval: `eval.rs:1163` — `Value::Builtin("error", 1)` → runtime panic с сообщением
- Загружается через `builtin_env()` в eval и type checker

### Prelude mechanism

- `include_str!("../../../prelude/prelude.sno")` в eval (lib.rs:104) и codegen (lib.rs:38)
- `prepend_prelude(source)` — конкатенирует prelude + user source перед парсингом
- Работает для interpreter и JIT
- Shadowing естественный: user code идёт после prelude, определения перекрывают

### Pipe-friendly API
```sno
-- Уже работает:
Ok 5 |> map_ok (\n -> n * 2) |> unwrap_or 0   -- == 10
```

## Что осталось сделать

### Расширение prelude (новые комбинаторы)

Для удобства больших программ полезно добавить:

```sno
--- Apply function to Ok value, return default on Err.
fold_result : (a -> b) -> b -> Result a e -> b
fold_result f _ (Ok x)  = f x
fold_result _ def (Err _) = def

--- Collect: list of Results -> Result of list (first Err wins).
sequence_results : [Result a e] -> Result [a] e
sequence_results [] = Ok []
sequence_results ((Err e):_) = Err e
sequence_results ((Ok x):rest) =
  sequence_results rest |> map_ok (\xs -> x : xs)

--- Maybe-style: convert to optional (drop error info).
ok_or_none : Result a e -> Maybe a
-- Требует Maybe в prelude (вне scope этой спеки)
```

**Decision:** `fold_result` и `sequence_results` добавить. `ok_or_none` — отложить до Maybe.

### Документация

| Что обновить | Содержимое |
|-------------|-----------|
| `docs/llm/synoema.md` | Уже есть? Проверить актуальность |
| `docs/llm/stdlib.md` | Добавить Result + комбинаторы |
| `docs/user/README.md` | Упомянуть error handling |

## Что НЕ входит

- `try/catch` — не ложится на чистую парадигму
- `?` operator (Rust-style early return) — может быть позже
- `Maybe a = Just a | None` — отдельная задача (None уже в builtins как конструктор)
- `Either a b` — Result покрывает этот use case
