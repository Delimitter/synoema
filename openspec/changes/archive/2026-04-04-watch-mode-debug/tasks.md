# Watch Mode — Tasks

## Tasks

- [x] 1. Refactor `run_file` и `jit_file` — выделить result-returning варианты (`run_file_inner`, `jit_file_inner`) которые возвращают `Result<(), ()>` вместо вызова `process::exit`. Существующие функции становятся обёртками.
- [x] 2. Реализовать `watch_loop` — основная функция: polling mtime, перезапуск, debug header, обработка ошибок без exit. Параметры: subcommand, path, interval, clear flag, error format.
- [x] 3. Реализовать `collect_watched_files` — сбор путей файла + его импортов через parse AST. Для `test` — все .sno файлы в директории.
- [x] 4. Добавить CLI ветку `"watch"` — парсинг аргументов (subcommand, file, --interval, --clear/--no-clear), вызов `watch_loop`.
- [x] 5. Обновить `--help` — добавить watch в список команд + watch options секция.
- [x] 6. Добавить тесты — 5 unit tests: collect_watched (single, import, dir), get_max_mtime, chrono_free_timestamp.
- [x] 7. Обновить документацию — `CLAUDE.md` (секция Команды).
