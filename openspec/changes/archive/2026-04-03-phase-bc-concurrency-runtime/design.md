# Design: Phase B + C Implementation

## D1: `spawn` outside `scope` — drop or error?

**Решение:** Drop the handle (thread runs detached, no join guarantee).

**Альтернативы:**
- Panic at runtime: слишком жёстко для небольшой ошибки
- Global fallback scope: усложняет модель

**Обоснование:** Structured concurrency model подразумевает, что spawn без scope — programmer error. Мы логируем debug-варнинг, но не паникуем. Для тестирования это не критично: тесты всегда используют scope.

## D2: `chan` как 0-arity builtin vs keyword

**Решение:** 0-arity builtin (как `readline`).

**Альтернативы:**
- Keyword: требует изменений в лексере, парсере, многих местах
- 1-arity `chan unit`: неудобнее для LLM-генерации

**Обоснование:** `readline` прецедент. `chan` вызывается немедленно при lookup — каждый `ch = chan` создаёт свежий канал. Это корректная семантика: `ch` связывается с конкретным каналом, все дальнейшие использования `ch` ссылаются на тот же канал.

## D3: `ChanInner` — Mutex vs RwLock

**Решение:** `Mutex<Sender<Value>>` + `Mutex<Receiver<Value>>`.

**Альтернативы:**
- RwLock: не нужен (sender и receiver всегда exclusive operations)
- SyncSender (bounded): может deadlock если буфер переполнен

**Обоснование:** `Sender<Value>` является Clone-able, но нам нужен один sender для простоты. Mutex overhead minimal по сравнению с thread spawning.

## D4: `Value: Send` — отсутствие явного impl

**Решение:** Полагаться на автоматический derive.

`Value` содержит только owned types (String, Vec, Arc). Нет Rc, RefCell, raw pointers. Rust автоматически выводит `Value: Send + Sync`.

**Риск:** Если в будущем добавить non-Send тип в Value — это сломает компиляцию с понятным сообщением об ошибке. Это хорошо: safety is checked at compile time.

## D5: Thread-local `SCOPE_STACK` — stack vs single vec

**Решение:** Stack of Vec (`Vec<Vec<JoinHandle<()>>>`).

**Обоснование:** Поддерживает вложенные scope корректно:
```
scope {              -- push []
  scope {            -- push []
    spawn f          -- → inner scope gets handle
  }                  -- pop inner, join 1 thread
  spawn g            -- → outer scope gets handle
}                    -- pop outer, join 1 thread
```

Single vec с индексами был бы fragile при early exit (Err).

## D6: JIT spawn — sequential stub vs real concurrency

**Решение (Phase B):** Sequential stub. `spawn e` в JIT компилируется как вычисление `e` без запуска потока.

**Причина Phase B:**
1. JIT spawn требует компиляции `e` как отдельной Cranelift-функции (thunk)
2. Передача среды выполнения (free variables) в thunk — нетривиально
3. Phase B фокусируется на interpreter-correctness

**Plan Phase C:** Настоящий JIT spawn через `synoema_jit_spawn_thunk(fn_ptr: *const u8)`. Spawn-выражение компилируется как отдельная `task_N() -> i64` функция.

## D7: Core IR — добавить `Spawn`/`Scope`

**Решение:** Добавить `CoreExpr::Spawn(Box<CoreExpr>)` и `CoreExpr::Scope(Box<CoreExpr>)` в Phase B уже (для корректной работы JIT pipeline без паники).

**Альтернатива:** Не добавлять, и крашиться если JIT встречает spawn/scope.

**Обоснование:** Добавить два варианта с sequential JIT stub дешевле, чем обрабатывать ошибку "unimplemented". Phase C заменит stub на настоящую реализацию.

## D8: `Chan a` — type constructor в HM

**Решение:** `Chan` как `Type::Con("Chan")`, `Chan a` = `Type::App(Con("Chan"), Var(a))`.

Это стандартный паттерн — аналогично `List a = App(Con("List"), Var(a))`. Никаких изменений в type unification не нужно: App unification уже работает.

**Display:** `Chan Int`, `Chan String`, `Chan a` — отображается через App Display.

## D9: JIT Chan — tagged pointer scheme

**Решение:** `ChanNode` на куче с tag `0x08` (следующий незанятый tag после Float 0x04).

```
i64 tagged value:
  bit 0 = 1: list pointer
  bit 1 = 1: string pointer
  bits 2-3 = 0x1 (tag=0x04): float pointer
  bits 2-3 = 0x2 (tag=0x08): chan pointer  ← новый
  0: int (raw)
```

Wait — надо проверить фактическую схему тегирования в runtime.rs, т.к. она может отличаться. Финальный tag будет выбран при изучении кода в процессе реализации.

## D10: Linearность spawn-выражений

**Решение:** Linear variables могут использоваться внутри `spawn expr` — один раз (по правилу linear-in-expr). Но linear variable нельзя использовать и в основном потоке, и в spawn (это было бы дублирование).

Для Phase B: `check_linear_in_expr` для `Spawn(e)` и `Scope(body)` обрабатываются как обычные выражения (проходим внутрь).

## Затронутые файлы

| Файл | Phase | Изменение |
|------|-------|-----------|
| `synoema-lexer/src/token.rs` | B | `KwScope`, `KwSpawn` |
| `synoema-lexer/src/scanner.rs` | B | scan scope/spawn |
| `synoema-parser/src/ast.rs` | B | `Scope`, `Spawn` ExprKind |
| `synoema-parser/src/parser.rs` | B | parse scope/spawn |
| `synoema-types/src/infer.rs` | B+C | typing Scope/Spawn; Chan type + builtins |
| `synoema-core/src/core_ir.rs` | B | `CoreExpr::Spawn`, `CoreExpr::Scope` |
| `synoema-core/src/desugar.rs` | B | desugar Spawn/Scope |
| `synoema-eval/src/value.rs` | C | `ChanInner`, `Value::Chan` |
| `synoema-eval/src/eval.rs` | B+C | SCOPE_STACK; chan/send/recv builtins |
| `synoema-codegen/src/runtime.rs` | C | chan FFI functions |
| `synoema-codegen/src/compiler.rs` | C | compile Spawn/Scope/Chan |
| Tests | B+C | ≥16 новых тестов |
