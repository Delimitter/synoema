# Design: IO/Effects System

## Architecture Decisions

### D1: IO как Type::App, не отдельный variant

**Решение:** `IO a` представляется как `Type::App(Box<Type::Con("IO")>, Box<a>)`.

**Альтернативы:**
- `Type::IO(Box<Type>)` — отдельный variant. Проще, но загрязняет Type enum.
- Effect row types — слишком инвазивно.

**Обоснование:** `Type::App` уже используется для `List a`, `Chan a`. IO — аналогичный type constructor. Минимальное изменение в types.rs.

```rust
// Новый helper:
impl Type {
    pub fn io(inner: Type) -> Self {
        Type::App(Box::new(Type::Con("IO".into())), Box::new(inner))
    }
}
```

### D2: Фаза 1 — мягкое IO (annotation-only, no enforcement)

**Решение:** IO типы аннотируются на builtins, но type checker НЕ запрещает вызов IO из pure-контекста. Это Phase 1 — формализация.

**Почему НЕ enforcement сейчас:**
1. Существующий код (`f x = print (show x)`, spawn-тесты) сломается
2. Enforcement требует io_context propagation через HOF (map print xs) — ~500 LOC
3. Формализация типов — уже ценна: LLM видит `IO ()` в сигнатуре и понимает, что функция impure

**Future Phase:** Strict enforcement (opt-in, через compiler flag `--strict-io`).

### D3: IO auto-unwrap — implicit coercion

**Решение:** `IO a` автоматически coerce в `a` при application. Type checker при унификации `IO a ~ a` unwrap IO.

```rust
// В unify:
// IO a ~ a → Ok (unwrap IO)
// a ~ IO a → Ok (wrap to IO)
```

Это позволяет:
```
main = print 42          -- print : a -> IO (), результат IO ()
main = print 42 ; 0      -- ; unwrap'ит IO () → (), возвращает 0 : Int
name = readline           -- readline : IO String, name : String
```

### D4: Builtins retyping

Все IO builtins получают `IO` в return type:

```rust
// print: ∀a. a -> IO ()  (было: a -> ())
// readline: IO String     (было: String)
// file_read: String -> IO String
// tcp_listen: Int -> IO Int
// tcp_accept: Int -> IO Int
// fd_readline: Int -> IO String
// fd_write: Int -> String -> IO ()
// fd_close: Int -> IO ()
// fd_popen: String -> IO Int
```

Pure builtins без изменений: `show`, `length`, `head`, `tail`, `map`, `filter`, math builtins.

### D5: Type parsing — IO как UpperId

`IO` парсится как `UpperId` token (уже существующий). Type application `IO a` парсится через `parse_type_app`. Ничего менять в parser не нужно — проверить что `IO ()`, `IO Int`, `String -> IO ()` работают.

### D6: Display для IO типов

```rust
// Type::App(Con("IO"), inner) → "IO inner"
// Уже работает через существующий Display для App
```

### D7: Core IR — IO erased

Core IR не знает про IO. Десахаризация стирает IO-обёртку. Eval и codegen не затрагиваются.

### D8: Unify IO coercion

При унификации:
- `IO a ~ IO b` → `unify(a, b)` (стандартное App unification)
- `IO a ~ a` → Ok, unwrap (мягкий режим)
- `a ~ IO a` → Ok, wrap (мягкий режим)

Реализация: в `unify`, при Type::App с "IO", если другая сторона не App("IO", _), пробуем unwrap.

## Изменённые файлы

| Файл | Изменение |
|------|-----------|
| `synoema-types/src/types.rs` | `Type::io()` helper |
| `synoema-types/src/infer.rs` | IO builtin types |
| `synoema-types/src/unify.rs` | IO coercion при унификации |
| `synoema-types/src/tests.rs` | Тесты IO type |
| `docs/llm/io.md` | IO documentation |
| `docs/specs/language_reference.md` | IO spec |

## Не меняется

- `synoema-lexer/` — IO = UpperId, новых токенов нет
- `synoema-parser/` — IO a парсится через существующий type app
- `synoema-core/` — IO erased at desugar
- `synoema-eval/` — без изменений
- `synoema-codegen/` — без изменений
- `synoema-repl/` — без изменений
