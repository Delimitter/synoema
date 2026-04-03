# Spec: IO Type Constructor

## Type System

```
IO : Type -> Type
```

`IO a` — тип, обозначающий вычисление с побочными эффектами, возвращающее `a`.

### Правила

1. **IO — тип-конструктор, не монада.** Нет bind (>>=), нет return. IO-значения выполняются при вызове.
2. **Pure ⊄ IO.** Pure-функция не может вызвать IO-функцию. IO-функция может вызвать pure.
3. **main — IO-контекст.** Тело `main` имплицитно разрешает IO.
4. **Existing builtins retyped:**

| Builtin | Старый тип | Новый тип |
|---------|-----------|-----------|
| print | `a -> ()` | `a -> IO ()` |
| readline | `String` | `IO String` |
| file_read | `String -> String` | `String -> IO String` |
| tcp_listen | `Int -> Int` | `Int -> IO Int` |
| tcp_accept | `Int -> Int` | `Int -> IO Int` |
| fd_readline | `Int -> String` | `Int -> IO String` |
| fd_write | `Int -> String -> ()` | `Int -> String -> IO ()` |
| fd_close | `Int -> ()` | `Int -> IO ()` |
| fd_popen | `String -> Int` | `String -> IO Int` |

5. **IO-контекст propagation:** Если функция `f` вызывает IO-builtin, `f` должна иметь IO в возвращаемом типе.
6. **Implicit unwrap в IO-контексте:** В теле IO-функции результат `IO a` автоматически unwrap до `a`.

## Effect Checking

```
Γ ⊢ e : IO a    context = IO
────────────────────────────
Γ ⊢ e : a        (auto-unwrap)

Γ ⊢ e : IO a    context = Pure
────────────────────────────────
TypeError: IO operation in pure context
```

## Синтаксис

```
-- IO-аннотированная функция (type signature)
greet : String -> IO ()
greet name = print ("Hello " ++ name)

-- main — имплицитно IO
main = greet "World"

-- pure функция — нет IO в сигнатуре
add x y = x + y

-- Ошибка: pure вызывает IO
bad x = print x + 1    -- TypeError: IO in pure context
```

## Не меняется

- Runtime: IO — compile-time маркер, не runtime обёртка
- Codegen: JIT не видит IO (erased after typecheck)
- Core IR: без изменений (IO erased при десахаризации)
- Eval: без изменений (уже выполняет IO напрямую)
