# Spec: project-init

## CLI Behaviour

```
synoema init [NAME] [--force] [--no-git]
```

| Invocation | Behaviour |
|---|---|
| `synoema init myapp` | Create `./myapp/` with full scaffold |
| `synoema init` | Scaffold current directory (name = basename of cwd) |
| `synoema init myapp --force` | Scaffold even if `myapp/` non-empty |
| `synoema init myapp --no-git` | Skip `.gitignore` |
| `synoema init --help` | Show init-specific usage |

**Error conditions:**
- Target directory exists and is non-empty → exit 1, message: `"'<dir>' already exists and is not empty. Use --force to overwrite."`
- File system write error → exit 1, propagate OS error message

## Generated File Tree

```
<name>/
├── src/
│   └── main.sno
├── tests/
│   └── test.sno
├── project.sno
├── CLAUDE.md
└── .gitignore          ← omitted with --no-git
```

## Template Substitutions

All templates support one substitution token: `{{name}}` → project name.

### `src/main.sno`
```
main = print "Hello, {{name}}!"
```

### `tests/test.sno`
```
--- test: greeting
--- expect: "Hello, {{name}}!"
main = print "Hello, {{name}}!"
```

### `project.sno`
```
name = "{{name}}"
version = "0.1.0"
entry = "src/main.sno"
```

### `CLAUDE.md`
```markdown
# {{name}}

Synoema project. Entry: `src/main.sno`.

## Commands
synoema run src/main.sno   -- run
synoema test tests/        -- test
synoema doc src/           -- docs

## Language
See: https://github.com/synoema/synoema/blob/main/docs/llm/synoema.md
```

### `.gitignore`
```
target/
*.snc
.DS_Store
```

## Implementation Spec

**File:** `lang/crates/synoema-repl/src/main.rs`

Add arm to `match positional.first().copied()`:
```rust
Some("init") => {
    let name_or_dot = positional.get(1);
    let force = positional.iter().any(|a| *a == "--force");
    let no_git = positional.iter().any(|a| *a == "--no-git");
    init_project(name_or_dot.copied(), force, no_git);
}
```

**Templates location:** `lang/templates/` — embedded with `include_str!`.

**Substitution:** simple `str.replace("{{name}}", name)` — no external templating crate.

## No New Dependencies

Only `std::fs::create_dir_all`, `std::fs::write`. No crate additions.
