# Proposal: MCP Documentation, Versioning Policy, and Binary Releases

## Problem Statement

1. **MCP-сервер без документации** — `mcp/synoema-mcp` собирается и работает, но нет ни одного документа, описывающего как его собрать, установить и подключить к Claude Desktop / другим MCP-клиентам. README.md не упоминает MCP.

2. **Отсутствует политика версий** — проект находится в alpha-стадии (версия `0.1.0`), но нигде не документировано, что это alpha, как нумеруются версии, что гарантируется между версиями, когда наступит beta/stable.

3. **Нет бинарных релизов** — пользователи вынуждены собирать из исходников (требует Rust toolchain). Нет структуры для хранения и распространения pre-built бинарников по платформам.

## Решение

### 1. `docs/mcp.md` — документация MCP сервера
- Что такое MCP-сервер Synoema, какие tools/resources предоставляет
- Сборка из исходников (`cargo build`)
- Установка (`cargo install`)
- Подключение к Claude Desktop (claude_desktop_config.json)
- Примеры использования инструментов eval/typecheck/run

### 2. `docs/versioning.md` — политика версионирования
- Текущая версия: `0.1.0-alpha.1`
- Схема: `MAJOR.MINOR.PATCH-STAGE.N`
- Alpha: нет гарантий стабильности API/синтаксиса
- Beta: стабильный синтаксис, возможны изменения stdlib
- Stable (1.0): полная обратная совместимость
- Добавить badge в README.md, упоминание в PROJECT_STATE.md

### 3. `releases/` — структура для бинарных релизов
- `releases/v0.1.0-alpha.1/{platform}/` — директории по платформам
- Платформы: darwin-arm64, darwin-x64, linux-x64, win32-x64
- Каждая директория: README с инструкциями по запуску
- `releases/README.md` — общий обзор + ссылки на платформы
- `.github/workflows/release.yml` — CI для сборки и публикации бинарников
- `docs/install.md` — пошаговые инструкции для пользователей

## Out of Scope
- Фактическая сборка бинарников (требует CI)
- npm-обёртка (отдельная задача)
- Автоматическая публикация на GitHub Releases (требует secrets)
