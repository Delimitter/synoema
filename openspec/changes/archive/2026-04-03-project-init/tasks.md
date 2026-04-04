# Tasks: project-init

## T1. Template files

- [ ] Создать `lang/templates/main.sno.tmpl` — hello world с `{{name}}`
- [ ] Создать `lang/templates/test.sno.tmpl` — пример теста
- [ ] Создать `lang/templates/project.sno.tmpl` — манифест
- [ ] Создать `lang/templates/CLAUDE.md.tmpl` — LLM context
- [ ] Создать `lang/templates/gitignore.tmpl` — .gitignore

## T2. `init_project` function in main.rs

- [ ] Добавить константы `include_str!` для всех шаблонов
- [ ] Реализовать `fn init_project(name_arg: Option<&str>, force: bool, no_git: bool)`
- [ ] `determine_project_name` — из аргумента или basename cwd
- [ ] `determine_root_path` — если имя задано → `./name/`, иначе cwd
- [ ] Проверка непустой директории → exit 1 без --force
- [ ] `std::fs::create_dir_all` для `src/` и `tests/`
- [ ] Запись всех файлов с подстановкой `{{name}}`
- [ ] `--no-git` — пропустить `.gitignore`
- [ ] Вывод `Created Synoema project '<name>'` + next steps

## T3. CLI wiring

- [ ] Добавить `Some("init")` arm в `match positional.first().copied()`
- [ ] Добавить `init` в `--help` output
- [ ] Обновить docstring модуля в начале `main.rs`

## T4. Tests

- [ ] Тест: `init_creates_structure` — init в tempdir, все файлы существуют
- [ ] Тест: `init_runs_main` — `synoema run src/main.sno` выводит `Hello, <name>!`
- [ ] Тест: `init_fails_nonempty` — init в непустой директории → exit 1
- [ ] Тест: `init_force_nonempty` — `--force` в непустой → успех
- [ ] Тест: `init_no_git` — `--no-git` → нет `.gitignore`

## T5. Documentation

- [ ] Обновить `CLAUDE.md` — добавить `synoema init` в секцию Команды
- [ ] Обновить `docs/user/README.md` — quickstart с init
- [ ] Обновить `context/PROJECT_STATE.md` — добавить `synoema init` в CLI

## Verify

- [ ] `cargo test` — 0 failures, 0 warnings
- [ ] `synoema init testproj && synoema run testproj/src/main.sno` → `Hello, testproj!`
