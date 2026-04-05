# Watch Mode for Debug

## Why

При отладке .sno файлов разработчик вынужден вручную перезапускать `synoema run` после каждого изменения. Это нарушает flow — переключение окон, набор команды, ожидание. Watch mode автоматически перезапускает при изменении файла, показывая результат мгновенно.

## What Changes

- Новая CLI команда `synoema watch <subcommand> <file> [options]`
- Поддерживает подкоманды: `run`, `jit`, `build`, `test`
- Polling на основе `std::fs::metadata` (mtime) — без новых зависимостей
- Отладочный вывод: timestamp, путь файла, время выполнения, номер перезапуска
- Ctrl+C для выхода (стандартный SIGINT)

## Capabilities

### New Capabilities

- `cli-watch-mode`: команда `watch` для автоматического перезапуска при изменении файла
- `cli-watch-debug-info`: отладочная информация (timing, run counter, file path) при каждом запуске

### Modified Capabilities

- Нет модификаций существующих команд

## Impact

- **Код**: `lang/crates/synoema-repl/src/main.rs` — новая команда `watch` + функция `watch_loop`
- **API**: 1 новая CLI команда (не ломает существующие)
- **Зависимости**: нет новых (std only: `std::fs::metadata`, `std::thread::sleep`, `std::time`)
- **Платформы**: кроссплатформенный (mtime через std)
