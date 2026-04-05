# Watch Mode — Design

## Architecture

Одна функция `watch_loop` в `main.rs`, вызываемая из CLI match-ветки `"watch"`.

### Поток данных

```
CLI args → parse subcommand + file + options
         → initial run (execute subcommand)
         → loop:
             sleep(interval)
             check mtime(file) + mtime(imports)
             if changed → clear screen → re-run → print debug header
```

### Ключевые решения

1. **Polling vs inotify/kqueue**: Polling через `std::fs::metadata().modified()`. Причина: правило минимализма запрещает новые зависимости. 500ms polling — приемлемый latency для отладки.

2. **Отслеживание импортов**: Для `run` и `jit` — после первого parse нужно собрать пути импортированных файлов и отслеживать их mtime тоже. Используем `synoema_parser::parse` + обход AST для `use` деклараций.

3. **Refactor run_file/jit_file**: Текущие функции вызывают `process::exit(1)` при ошибках. Для watch нужен вариант, который возвращает `Result` вместо exit. Решение: выделить `run_file_result` / `jit_file_result`, а существующие `run_file` / `jit_file` станут тонкими обёртками.

4. **Очистка экрана**: ANSI escape `\x1b[2J\x1b[H` (кроссплатформенный — работает на macOS, Linux, Windows Terminal). Отключается через `--no-clear`.

5. **Graceful shutdown**: `std::sync::atomic::AtomicBool` + `ctrlc`-подобный паттерн через `std::process` — нет, просто стандартный SIGINT убивает процесс, этого достаточно.

## File Changes

| Файл | Изменение |
|------|-----------|
| `lang/crates/synoema-repl/src/main.rs` | Новая CLI ветка `"watch"`, функция `watch_loop`, refactor `run_file`/`jit_file` в result-returning варианты |

## Import Tracking

Для отслеживания импортов:
```rust
fn collect_watched_files(path: &str) -> Vec<PathBuf> {
    let mut files = vec![PathBuf::from(path)];
    // Parse file, extract `use` declarations, resolve paths
    if let Ok(source) = std::fs::read_to_string(path) {
        if let Ok(program) = synoema_parser::parse(&source) {
            for decl in &program.declarations {
                // extract import paths, resolve relative to base_dir
            }
        }
    }
    files
}
```

## mtime Tracking

```rust
fn get_max_mtime(files: &[PathBuf]) -> Option<SystemTime> {
    files.iter()
        .filter_map(|f| std::fs::metadata(f).ok()?.modified().ok())
        .max()
}
```
