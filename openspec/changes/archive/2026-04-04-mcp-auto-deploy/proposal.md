# MCP Auto-Deploy

## Why

Сейчас `synoema init` генерирует MCP-конфиги с `npx synoema-mcp`, но:

1. **npx требует Node.js** — в Docker/CI/embedded-окружениях Node.js может отсутствовать
2. **Нет version pinning** — `npx synoema-mcp` всегда тянет latest (или кешированную), нет контроля версии
3. **Нет fallback-стратегии** — если npx недоступен, пользователь вручную ищет бинарник
4. **Нет `--version` / health check** — MCP-сервер не поддерживает `--version`, нельзя программно проверить версию
5. **Нет механизма обновления** — ни встроенного уведомления, ни update-команды
6. **Build from source не интегрирован** — Makefile-ы лежат в releases/, но init про них не знает

## What Changes

- MCP-сервер поддерживает `--version` и `--health` CLI-флаги
- `synoema init` генерирует MCP-конфиги с version pinning (`npx synoema-mcp@0.1.0-alpha.1`)
- Новая команда `synoema mcp-install` для standalone-установки бинарника (без Node.js)
- `synoema mcp-install` определяет ОС/архитектуру и скачивает правильный бинарник с GitHub Releases
- Fallback: если npx недоступен → конфиг указывает на локальный бинарник
- `synoema mcp-update` проверяет новую версию и обновляет
- Генерируемые конфиги адаптированы под метод установки (npx vs binary path)

## Capabilities

### New Capabilities

- `mcp-version-flag`: CLI-флаг `--version` для MCP-сервера
- `mcp-health-flag`: CLI-флаг `--health` для MCP-сервера (JSON health check)
- `mcp-install-command`: Команда `synoema mcp-install` — автоустановка бинарника
- `mcp-update-command`: Команда `synoema mcp-update` — проверка и обновление
- `mcp-version-pinning`: `synoema init` пинит версию в MCP-конфигах

### Modified Capabilities

- `init-project`: Адаптация init для выбора deployment-стратегии (npx vs binary)

## Impact

- **Код**: `mcp/synoema-mcp/src/main.rs` (CLI-флаги), `lang/crates/synoema-repl/src/main.rs` (mcp-install, mcp-update, init), `lang/templates/` (шаблоны с версией)
- **API**: Новые CLI-команды `mcp-install`, `mcp-update`; новые флаги `--version`, `--health` для MCP-бинарника
- **Зависимости**: нет новых — HTTP-запросы через `std::io` + `std::process::Command` (curl/wget fallback)
- **Платформы**: darwin-arm64, darwin-x64, linux-x64, win32-x64
