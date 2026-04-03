# Proposal: Phase B + C — Structured Concurrency Runtime and Typed Channels

## Problem Statement

Phase A добавил линейные типы как фундамент безопасности данных в конкурентном коде. Теперь нужны конструкции выполнения:

1. **Phase B**: `scope { spawn expr }` — структурированная конкурентность: OS-потоки с гарантированным временем жизни
2. **Phase C**: `chan` / `send` / `recv` — типизированные каналы для межпоточной коммуникации

Без этих примитивов LLM не может генерировать параллельный Synoema-код.

## Design Principles (из proposal фазы A)

- **Structured concurrency**: scope гарантирует, что потоки не переживают родителя
- **Каналы с copy semantics**: arena-совместимы, нет shared state
- **Минимальный синтаксис**: каждый оператор = 1 BPE-токен
- **Зависимости**: только std (no tokio, no async)

## Phase B: Structured Concurrency Runtime

### Синтаксис
```
scope { spawn (f x) ; spawn (g y) }
```

- `scope` — ключевое слово, 1 BPE-токен ✓
- `spawn` — ключевое слово, 1 BPE-токен ✓
- `{ }` — уже существуют в лексере (LBrace/RBrace)
- `;` (Seq) — уже существует в BinOp::Seq

### Семантика
- `spawn expr` — запускает вычисление `expr` в новом OS-потоке, возвращает `Unit`
- `scope { body }` — создаёт scope, запускает `body`, ожидает все spawned потоки, возвращает результат `body`
- Структурированность: spawn вне scope — no-op (нет утечки потоков)

### Типы
- `spawn e : Unit` для любого `e : a`
- `scope e : a` где `e : a`

### Ограничения Phase B
- **Только interpreter** (JIT получает sequential stub)
- Spawned threads не разделяют IO-состояние с родителем (thread-local IO)
- Per-thread Evaluator (независимый evaluator для каждого потока)

## Phase C: Typed Channels

### Синтаксис (builtin-функции, не ключевые слова)
```
ch = chan           -- создать канал
send ch 42         -- отправить
x = recv ch        -- получить (блокирующий)
```

- `chan` — 0-arity builtin, 1 BPE-токен ✓
- `send` — 2-arity builtin, 1 BPE-токен ✓
- `recv` — 1-arity builtin, 1 BPE-токен ✓

### Типы
- `chan : ∀a. Chan a` (0-arity, fresh channel каждый раз)
- `send : ∀a. Chan a → a → Unit`
- `recv : ∀a. Chan a → a`

### Реализация
- `Value::Chan(Arc<ChanInner>)` — thread-safe, Clone через Arc
- `ChanInner`: `Mutex<Sender<Value>>` + `Mutex<Receiver<Value>>` (std::sync::mpsc)
- JIT: FFI calls через `synoema_chan_new/send/recv`
- JIT spawn: thunk-подход (`synoema_jit_spawn_thunk(fn_ptr)`)

## Полный пример
```
producer ch =
  send ch 1
  send ch 2
  send ch 3

consumer ch =
  x = recv ch
  y = recv ch
  z = recv ch
  print (show (x + y + z))

main =
  ch = chan
  scope {
    spawn (producer ch)
    consumer ch
  }
```

## Success Criteria

### Phase B
- [ ] `scope { spawn (f x) }` запускает f x в отдельном потоке
- [ ] scope дожидается всех spawned потоков
- [ ] Nested scopes работают корректно
- [ ] ≥8 тестов для scope/spawn
- [ ] 0 warnings, все существующие тесты проходят

### Phase C
- [ ] `chan` создаёт новый типизированный канал
- [ ] `send ch v` / `recv ch` работают в разных потоках
- [ ] Type checker: `Chan a` как type constructor
- [ ] JIT: chan/send/recv через FFI
- [ ] ≥8 тестов для chan/send/recv
- [ ] 0 warnings, все существующие тесты проходят
