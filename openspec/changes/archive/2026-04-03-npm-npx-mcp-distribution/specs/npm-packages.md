# Spec: npm Package Structure

## Файловая структура

```
npm/
  synoema-mcp/
    package.json          ← главный пакет
    bin/
      run.js              ← JS entrypoint (chmod +x)
  platforms/
    darwin-arm64/
      package.json        ← @synoema/mcp-darwin-arm64
      .gitkeep
    darwin-x64/
      package.json        ← @synoema/mcp-darwin-x64
      .gitkeep
    linux-x64/
      package.json        ← @synoema/mcp-linux-x64
      .gitkeep
    win32-x64/
      package.json        ← @synoema/mcp-win32-x64
      .gitkeep
```

`.gitkeep` — placeholder: бинарники в git не коммитятся, копируются CI при релизе.

## Главный пакет `npm/synoema-mcp/package.json`

```json
{
  "name": "synoema-mcp",
  "version": "0.1.0-alpha.1",
  "description": "MCP server for the Synoema programming language — eval, typecheck, run",
  "bin": { "synoema-mcp": "./bin/run.js" },
  "files": ["bin/"],
  "optionalDependencies": {
    "@synoema/mcp-darwin-arm64": "0.1.0-alpha.1",
    "@synoema/mcp-darwin-x64":   "0.1.0-alpha.1",
    "@synoema/mcp-linux-x64":    "0.1.0-alpha.1",
    "@synoema/mcp-win32-x64":    "0.1.0-alpha.1"
  },
  "engines": { "node": ">=16" },
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/Delimitter/synoema",
    "directory": "npm/synoema-mcp"
  },
  "keywords": ["mcp", "synoema", "llm", "language-server"]
}
```

## Платформенные пакеты `npm/platforms/{platform}/package.json`

Пример для `darwin-arm64`:
```json
{
  "name": "@synoema/mcp-darwin-arm64",
  "version": "0.1.0-alpha.1",
  "description": "Synoema MCP server binary — macOS Apple Silicon",
  "os": ["darwin"],
  "cpu": ["arm64"],
  "files": ["synoema-mcp"],
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/Delimitter/synoema",
    "directory": "npm/platforms/darwin-arm64"
  }
}
```

Отличия между платформами:
| Пакет | os | cpu | файл |
|-------|----|-----|------|
| darwin-arm64 | darwin | arm64 | synoema-mcp |
| darwin-x64 | darwin | x64 | synoema-mcp |
| linux-x64 | linux | x64 | synoema-mcp |
| win32-x64 | win32 | x64 | synoema-mcp.exe |

## JS-обёртка `npm/synoema-mcp/bin/run.js`

- `require.resolve()` для поиска бинарника из optional dependency
- `spawnSync` с `stdio: "inherit"` — передаёт stdin/stdout/stderr напрямую (важно для MCP stdio протокола)
- Корректный exit code
- Понятное сообщение при unsupported platform
