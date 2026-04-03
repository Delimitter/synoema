# Tasks: MCP Docs, Versioning, Binary Releases

## Checklist

### A. MCP документация

- [ ] A1: Прочитать `mcp/synoema-mcp/src/tools.rs` и `resources.rs` — составить список tools/resources
- [ ] A2: Создать `docs/mcp.md` — полная документация MCP сервера
- [ ] A3: Обновить `README.md` — добавить раздел "## MCP Server" со ссылкой на `docs/mcp.md`

### B. Политика версионирования

- [ ] B1: Создать `docs/versioning.md` — политика версий
- [ ] B2: Обновить `README.md` — добавить версию и стадию в шапку
- [ ] B3: Обновить `context/PROJECT_STATE.md` — раздел о текущей версии и стадии
- [ ] B4: Обновить `mcp/Cargo.toml` и `mcp/synoema-mcp/Cargo.toml` — version = "0.1.0-alpha.1"

### C. Каталог бинарных релизов

- [ ] C1: Создать `releases/README.md` — обзор структуры релизов
- [ ] C2: Создать `releases/v0.1.0-alpha.1/README.md` — changelog первого alpha
- [ ] C3: Создать README для каждой платформы (darwin-arm64, darwin-x64, linux-x64, win32-x64)
- [ ] C4: Создать `.github/workflows/release.yml` — CI workflow для сборки бинарников
- [ ] C5: Создать `docs/install.md` — пользовательская инструкция по скачиванию и запуску

---

## Детализация

### A1: Инвентаризация MCP tools/resources
Файлы: `mcp/synoema-mcp/src/tools.rs`, `resources.rs`, `prompts.rs`

### A2: `docs/mcp.md`
Разделы:
1. Overview — что такое Synoema MCP Server
2. Tools — eval, typecheck, run (с примерами)
3. Resources — language spec, grammar
4. Quick Start — через pre-built бинарник (ссылка на docs/install.md)
5. Build from source — `cd mcp && cargo build --release`
6. Install via cargo — `cargo install --path mcp/synoema-mcp`
7. Claude Desktop config — claude_desktop_config.json пример
8. Other MCP clients — cursor, zed, etc.

### A3: Обновление README.md
Вставить после раздела "## Structured Diagnostics":
```markdown
## MCP Server

Synoema включает MCP-сервер для интеграции с Claude Desktop и другими LLM-инструментами.
Подробная документация: [docs/mcp.md](docs/mcp.md)
```

### B1: `docs/versioning.md`
Разделы: текущая версия, схема нумерации, что означает alpha/beta/stable, гарантии по каждой стадии, roadmap к stable.

### B2: Обновление README.md шапки
Добавить строку `**Version: 0.1.0-alpha.1** — alpha stage, APIs may change.` после статус-блока.

### C3: Platform README файлы
Каждый содержит:
1. Ссылку на GitHub Releases для скачивания бинарника
2. chmod +x инструкцию (macOS/Linux)
3. macOS Gatekeeper обход (`xattr -dr com.apple.quarantine`)
4. Пример запуска `synoema jit examples/factorial.sno`
5. Пример запуска MCP сервера и конфиг Claude Desktop

### C4: `.github/workflows/release.yml`
Триггер: `push: tags: ['v*']`
Matrix build: 4 платформы
Steps: checkout → dtolnay/rust-toolchain → cargo build --release → upload to GitHub Release
