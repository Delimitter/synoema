# Spec: MCP Server Documentation

## Scope
Документация для `mcp/synoema-mcp` — MCP-сервер Synoema.

## Файл: `docs/mcp.md`

### Структура документа
1. Что такое Synoema MCP Server
2. Возможности (tools + resources)
3. Быстрый старт — предсобранный бинарник
4. Сборка из исходников
5. Установка через cargo install
6. Подключение к Claude Desktop
7. Подключение к другим MCP-клиентам
8. Примеры инструментов
9. Troubleshooting

### Tools, предоставляемые сервером
Из `mcp/synoema-mcp/src/tools.rs`:
- `eval` — вычислить выражение Synoema, вернуть результат
- `typecheck` — проверить типы кода Synoema
- `run` — выполнить файл .sno

### Resources
Из `mcp/synoema-mcp/src/resources.rs`:
- Language reference
- Spec/grammar

## Изменения в `README.md`
- Добавить раздел "## MCP Server" после "## Constrained Decoding"
- Краткое описание + ссылка на `docs/mcp.md`

## Требования к документу
- Примеры конфигурации claude_desktop_config.json
- Указание на платформу (darwin/linux/windows)
- Ссылка на `releases/` для скачивания бинарников
- Ссылка на `docs/versioning.md` для политики версий
