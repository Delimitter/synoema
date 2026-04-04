# Design: Project Init

## Реализация

Всё в `repl/src/main.rs`:
- Новая subcommand `Some("init")`
- Шаблоны через `include_str!` из `lang/templates/`
- Placeholder `{name}` заменяется через `str::replace`

## Files

| File | Change |
|------|--------|
| `repl/src/main.rs` | +`init_project()` function + CLI match |
| `lang/templates/main.sno.tmpl` | NEW |
| `lang/templates/test.sno.tmpl` | NEW |
| `lang/templates/project.sno.tmpl` | NEW |
| `lang/templates/CLAUDE.md.tmpl` | NEW |
| `lang/templates/gitignore.tmpl` | NEW |
