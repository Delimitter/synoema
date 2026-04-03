# Proposal: npm/npx Distribution для synoema-mcp

## Problem Statement

Synoema MCP сервер (`mcp/synoema-mcp`) сейчас доступен только через:
1. Сборку из исходников (требует Rust toolchain)
2. Pre-built бинарники из GitHub Releases (ручное скачивание)

Для MCP-клиентов (Claude Desktop, Cursor, Zed) наиболее удобный способ установки — через npm/npx. Например, `npx synoema-mcp` — одна команда без установки.

## Решение

Добавить в репозиторий `npm/` директорию по паттерну esbuild/Biome/Bun:

- Один платформенный пакет на target: `@synoema/mcp-darwin-arm64`, `@synoema/mcp-darwin-x64`, `@synoema/mcp-linux-x64`, `@synoema/mcp-win32-x64`
- Главный пакет `synoema-mcp` с JS-обёрткой и `optionalDependencies`
- CI шаг в `release.yml` для публикации на npmjs.com

## Из предыдущего изменения

В `.github/workflows/release.yml` уже есть matrix build бинарников. Нужно добавить npm publish шаги к существующему workflow.

## Out of Scope
- Пакет для `synoema` (compiler/REPL) — только MCP
- `@synoema/mcp-linux-arm64`, musl, FreeBSD — можно добавить позже
- Автоматический bump версии в package.json — вручную перед тегом
