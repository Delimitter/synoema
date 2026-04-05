# Design: MCP Auto-Deploy

## Context

Текущее состояние:
- MCP-сервер: Rust-бинарник, общается по stdio JSON-RPC
- Распространение: npm-пакет `synoema-mcp` с platform-specific optionalDependencies
- Init: `synoema init --ai <target>` генерирует конфиги с `npx synoema-mcp`
- CI/CD: `.github/workflows/release.yml` — собирает 4 платформы, публикует npm + GitHub Release
- Версия: `mcp/Cargo.toml` → `0.1.0-alpha.1`, baked into binary via `env!("CARGO_PKG_VERSION")`

### Анализ deployment-стратегий

| Стратегия | Плюсы | Минусы | ОС |
|-----------|-------|--------|-----|
| **npx** | Zero-install, always available | Нужен Node.js ≥16, холодный старт ~2с | Все |
| **npm -g** | Быстрый запуск после install | Нужен Node.js, ручное обновление | Все |
| **Binary (GitHub Release)** | Нет зависимостей, быстрый старт | Ручная загрузка, нет auto-update | Все 4 |
| **cargo install** | Всегда свежая, из исходников | Нужен Rust toolchain, долгая компиляция | Все |
| **Homebrew** (будущее) | Привычно для macOS | Нужен отдельный tap, только macOS/Linux | darwin, linux |

### Вывод

Нужны два основных пути: **npx** (для тех у кого есть Node.js) и **binary** (для всех остальных). Оба должны поддерживать version pinning и обновление.

## Goals / Non-Goals

**Goals:**
- `--version` и `--health` для MCP-бинарника (для скриптов и health checks)
- `synoema mcp-install` — кросс-платформенная установка бинарника без Node.js
- `synoema mcp-update` — обновление до последней версии
- Version pinning в init-конфигах
- Поддержка darwin-arm64, darwin-x64, linux-x64, win32-x64

**Non-Goals:**
- Homebrew/apt/winget пакетные менеджеры (будущее)
- Auto-update background daemon
- linux-arm64 (нет CI runner в текущем workflow)

## Decisions

### D1: Install path — `~/.synoema/bin/`

Бинарник ставится в `~/.synoema/bin/synoema-mcp`. Это:
- Не требует sudo/admin
- Кросс-платформенно (`%USERPROFILE%\.synoema\bin\` на Windows)
- Не конфликтует с npm/cargo install
- Легко находить и обновлять

Альтернатива: `/usr/local/bin` — требует sudo, не подходит для CI без прав.

### D2: Download-механизм — shell out to curl/wget/powershell

Synoema REPL — Rust-бинарник без HTTP-зависимостей. Добавлять reqwest/ureq ради одной команды — нарушение правила минимума зависимостей (`context/RULES.md`: только Cranelift + pretty_assertions).

Стратегия: `std::process::Command` вызывает:
- macOS/Linux: `curl -fsSL` → fallback `wget -qO-`
- Windows: `powershell -Command "Invoke-WebRequest"`

### D3: Version source — `env!("CARGO_PKG_VERSION")`

Версия MCP-сервера уже baked at compile time. REPL тоже берёт из `env!("CARGO_PKG_VERSION")`. При `mcp-install` скачивается та же версия что и REPL (version lockstep).

При `mcp-update` — запрос GitHub API `https://api.github.com/repos/Delimitter/synoema/releases/latest` → `tag_name` → download.

### D4: Version pinning в init — `synoema-mcp@{version}`

Шаблоны меняются:
- Текущее: `"args": ["synoema-mcp"]`
- Новое: `"args": ["synoema-mcp@{{version}}"]`

`{{version}}` подставляется из `env!("CARGO_PKG_VERSION")` at init time.

### D5: --mcp-binary flag в init

Новый флаг `--mcp-binary` для init:
- Без флага (default): `npx synoema-mcp@{version}` (как сейчас, но с версией)
- С флагом: `~/.synoema/bin/synoema-mcp` (абсолютный путь)

Шаблоны: один набор с `{{mcp_command}}` и `{{mcp_args}}` placeholder'ами.

### D6: MCP CLI flags — `--version` и `--health`

В `main.rs` до входа в JSON-RPC loop:
```rust
let args: Vec<String> = std::env::args().collect();
if args.contains(&"--version".to_string()) {
    println!("synoema-mcp {}", env!("CARGO_PKG_VERSION"));
    return;
}
if args.contains(&"--health".to_string()) {
    println!(r#"{{"status":"ok","version":"{}","protocol":"2024-11-05"}}"#, env!("CARGO_PKG_VERSION"));
    return;
}
```

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| GitHub API rate limit (60 req/hr anonymous) | Кешировать version check; mcp-update — явная команда, не автоматическая |
| curl/wget отсутствует | На macOS curl есть всегда; на Linux 99%; Windows — PowerShell. Error message с инструкцией |
| Broken download URL | URL формируется из tag + platform; validate после скачивания (file size > 0, execute permission) |
| Version mismatch REPL ↔ MCP | mcp-install скачивает версию matching REPL; mcp-update предупреждает если мажорная расходится |

## Migration Plan

1. MCP `--version`/`--health` — backward-compatible (новые CLI-флаги)
2. Init version pinning — только новые проекты (существующие не затрагиваются)
3. `mcp-install`/`mcp-update` — новые команды, ничего не ломают
4. Шаблоны с `{{version}}` — backward-compatible через `apply_tmpl`
