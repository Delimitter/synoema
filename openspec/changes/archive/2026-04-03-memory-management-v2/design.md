---
id: design
type: design
status: done
---

# Design: Memory Management v2

## Компонент 1: fd_open / fd_open_write для файлов

### Текущее состояние I/O

```
    fd_popen "cmd"     → fd (BufReader<ChildStdout>)     ✅
    fd_readline fd     → String (одна строка)             ✅
    fd_write fd s      → () (запись)                      ✅
    fd_close fd        → () (закрыть + drop)              ✅
    file_read "path"   → String (весь файл целиком)       ✅
    fd_open "path"     → fd (файл для чтения)             ❌ ОТСУТСТВУЕТ
    fd_open_write "p"  → fd (файл для записи)             ❌ ОТСУТСТВУЕТ
```

### Реализация

Используем существующие thread-local maps `IO_READERS` и `IO_WRITERS`:

```rust
// synoema-eval/src/eval.rs

// fd_open: открыть файл для чтения
"fd_open" => {
    let path = sval(&args[0])?;
    let file = std::fs::File::open(&path)
        .map_err(|e| err_io(format!("fd_open: {}: {}", path, e)))?;
    let reader = std::io::BufReader::new(file);
    let fd = next_fd();
    IO_READERS.with(|r| r.borrow_mut().insert(fd, Box::new(reader)));
    Ok(Value::Int(fd))
}

// fd_open_write: открыть файл для записи
"fd_open_write" => {
    let path = sval(&args[0])?;
    let file = std::fs::File::create(&path)
        .map_err(|e| err_io(format!("fd_open_write: {}: {}", path, e)))?;
    let writer = std::io::BufWriter::new(file);
    let fd = next_fd();
    IO_WRITERS.with(|w| w.borrow_mut().insert(fd, Box::new(writer)));
    Ok(Value::Int(fd))
}
```

Существующие `fd_readline`, `fd_write`, `fd_close` работают без изменений — они оперируют абстрактными fd из тех же maps.

### Пример использования

```sno
-- Построчная обработка (O(line_size) memory):
process_file path =
  fd = fd_open path
  process_lines fd 0

process_lines fd count =
  line = fd_readline fd
  ? line == "" -> fd_close fd |> \_ -> count
  : process_lines fd (count + 1)

main = process_file "huge_dataset.csv"
```

```
    Файл (5 GB)
    ┌──────────────────────────────┐
    │ line1\n                       │
    │ line2\n      fd_readline      │──▶ String (одна строка, ~100 bytes)
    │ line3\n      ──────────▶      │    │
    │ ...                           │    ▼ process → drop → next line
    └──────────────────────────────┘

    Peak memory: O(max_line_length), NOT O(file_size)
```

### JIT Support

Для JIT: регистрировать `fd_open` и `fd_open_write` как FFI-функции в `compiler.rs`, аналогично существующим `fd_popen`, `fd_readline`.

```rust
// compiler.rs — register FFI
self.declare_fn("synoema_fd_open", &[I64], I64);       // path_ptr → fd
self.declare_fn("synoema_fd_open_write", &[I64], I64);  // path_ptr → fd

// runtime.rs — extern "C" implementations
pub extern "C" fn synoema_fd_open(path_val: i64) -> i64 { ... }
pub extern "C" fn synoema_fd_open_write(path_val: i64) -> i64 { ... }
```

## Компонент 2: Arena Overflow Warning + Tracking

### Текущая проблема

```rust
// runtime.rs — current overflow path:
if new_offset > ARENA_SIZE {
    unsafe { alloc(Layout::from_size_align(size, align).unwrap()) }
    // ↑ Silent. Leaked. No tracking.
}
```

### Решение

```rust
struct Arena {
    ptr: *mut u8,
    offset: usize,
    // NEW:
    overflow_allocs: Vec<(*mut u8, Layout)>,  // tracked overflow allocations
    overflow_warned: bool,                     // warn once per reset cycle
}

fn alloc(&mut self, size: usize, align: usize) -> *mut u8 {
    let aligned_abs = (base + align - 1) & !(align - 1);
    let new_offset = (aligned_abs - self.ptr as usize) + size;

    if new_offset > ARENA_SIZE {
        // WARNING (once per cycle)
        if !self.overflow_warned {
            eprintln!(
                "[synoema] arena overflow: {} bytes requested, \
                 {} / {} used. Falling back to system allocator.",
                size, self.offset, ARENA_SIZE
            );
            self.overflow_warned = true;
        }

        let layout = Layout::from_size_align(size, align).unwrap();
        let ptr = unsafe { alloc(layout) };

        // TRACK for cleanup
        self.overflow_allocs.push((ptr, layout));

        ptr
    } else {
        self.offset = new_offset;
        aligned_abs as *mut u8
    }
}
```

## Компонент 3: arena_save / arena_restore

Per-scope reset для серверных циклов в JIT.

```rust
// runtime.rs

/// Save current arena offset for later restore
pub extern "C" fn arena_save() -> i64 {
    ARENA.with(|a| a.borrow().offset as i64)
}

/// Restore arena to previously saved offset
/// WARNING: all pointers allocated after save become invalid
pub extern "C" fn arena_restore(saved: i64) {
    ARENA.with(|a| {
        let mut arena = a.borrow_mut();
        let saved = saved as usize;
        // Don't restore beyond current offset (safety)
        if saved <= arena.offset {
            arena.offset = saved;
        }
    });
}
```

### Server Pattern

```
    JIT Server Loop
    ═══════════════

    main = server_loop (tcp_listen 8080)

    server_loop sock =
      client = tcp_accept sock
      saved = arena_save ()         -- mark
      handle_request client         -- allocates in arena
      arena_restore saved           -- free request data
      server_loop sock              -- repeat (TCO)

    Arena: 0 ──▶ N ──▶ 0 ──▶ N ──▶ 0 (stable)
```

Для interpreter'а `arena_save` / `arena_restore` — no-op (нет арены).

## Компонент 4: Overflow Cleanup при arena_reset

```rust
fn reset(&mut self) {
    self.offset = 0;
    self.overflow_warned = false;

    // NEW: free all tracked overflow allocations
    for (ptr, layout) in self.overflow_allocs.drain(..) {
        unsafe { dealloc(ptr, layout); }
    }
}
```

### Взаимодействие компонентов

```
    ┌─────────────────────────────────────────────────┐
    │                    Arena                          │
    │                                                   │
    │  [──────── 8 MB bump region ──────────]           │
    │                    │                              │
    │              arena_save/restore                   │
    │              (per-scope cleanup)                  │
    │                                                   │
    │  Overflow:  [tracked Vec<(*mut, Layout)>]         │
    │              │                                    │
    │              └─── freed at arena_reset()          │
    │              └─── warning on first overflow       │
    └─────────────────────────────────────────────────┘

    File I/O:
    ┌─────────────────────────────────────────────────┐
    │  fd_open "file"  → BufReader in IO_READERS map   │
    │  fd_readline fd  → one line at a time            │
    │  fd_close fd     → drop reader, remove from map  │
    │                                                   │
    │  Memory: O(line_size) not O(file_size)            │
    └─────────────────────────────────────────────────┘
```

### Что остаётся не решённым (known limitations)

1. **Box::leak для string literals в JIT** — по-прежнему утекает. Решение: intern pool (отдельная задача).
2. **Box::into_raw для channels** — по-прежнему утекает. Решение: channel registry (отдельная задача, Phase D concurrency).
3. **Нет mmap / memory-mapped I/O** — для >RAM файлов. Не приоритет.
