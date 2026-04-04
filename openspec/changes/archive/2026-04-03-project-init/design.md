# Design: project-init

## Key Decisions

### 1. Template embedding — `include_str!`

Шаблоны встраиваются в бинарник как строковые константы. Нет runtime-зависимости от файловой системы для шаблонов.

```rust
const TMPL_MAIN: &str = include_str!("../../templates/main.sno.tmpl");
const TMPL_TEST: &str = include_str!("../../templates/test.sno.tmpl");
const TMPL_PROJECT: &str = include_str!("../../templates/project.sno.tmpl");
const TMPL_CLAUDE: &str = include_str!("../../templates/CLAUDE.md.tmpl");
const TMPL_GITIGNORE: &str = include_str!("../../templates/gitignore.tmpl");
```

Альтернатива (runtime template dir) — отклонена: нарушает принцип самодостаточного бинарника.

### 2. Template substitution — `str::replace`

Один токен `{{name}}`. Никакого движка шаблонов — нет зависимостей. `s.replace("{{name}}", name)` достаточно.

### 3. init_project — отдельная функция в main.rs

```rust
fn init_project(name_arg: Option<&str>, force: bool, no_git: bool) {
    let name = determine_project_name(name_arg);
    let root = determine_root_path(name_arg, &name);
    check_empty_or_force(&root, force);
    create_structure(&root, &name, no_git);
    print_success(&name, &root);
}
```

Не выносить в отдельный crate — функция ~60 строк, один файл.

### 4. Проверка непустой директории

```rust
fn is_dir_non_empty(path: &Path) -> bool {
    path.exists() && path.read_dir().map(|mut d| d.next().is_some()).unwrap_or(false)
}
```

При `--force`: создаём структуру поверх (не удаляем существующее, только дозаписываем).

### 5. Успешный вывод

После создания — вывод инструкции пользователю:
```
Created Synoema project 'myapp'

Next steps:
  cd myapp
  synoema run src/main.sno
```

### 6. `project.sno` — не Synoema-код

`project.sno` содержит простые key-value привязки, синтаксически валидные для парсера Synoema. Это позволит `synoema build` читать его через обычный `parse` в будущем.

## Files Changed

| File | Change |
|------|--------|
| `lang/crates/synoema-repl/src/main.rs` | Add `init` arm + `init_project` function |
| `lang/templates/main.sno.tmpl` | New file |
| `lang/templates/test.sno.tmpl` | New file |
| `lang/templates/project.sno.tmpl` | New file |
| `lang/templates/CLAUDE.md.tmpl` | New file |
| `lang/templates/gitignore.tmpl` | New file |
| `CLAUDE.md` | Add `synoema init` to commands |
| `docs/user/README.md` | Add quickstart with init |
| `context/PROJECT_STATE.md` | Update CLI commands section |

## No GBNF / BPE Changes

`init` is a subcommand name, not a language keyword. No BPE verification needed. No GBNF update.

## Test Strategy

Integration tests via `synoema test` framework:
1. `init_creates_structure` — run init in tempdir, check files exist
2. `init_runs_main` — init + `synoema run src/main.sno` produces expected output
3. `init_fails_nonempty` — init in non-empty dir exits 1
4. `init_force_nonempty` — `--force` succeeds in non-empty dir
5. `init_no_git` — `--no-git` omits `.gitignore`

Tests added as unit tests in `main.rs` using `#[cfg(test)]` + `tempdir` via `std::env::temp_dir()`.
