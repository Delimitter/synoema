# Design: npm/npx Distribution

## D1: `spawnSync` vs `execFileSync` в run.js

**Решение:** `spawnSync` с `stdio: "inherit"`.

**Почему важно:** MCP работает по stdio — JSON-RPC читается из stdin, пишется в stdout. `stdio: "inherit"` передаёт потоки напрямую без буферизации. `execFileSync` буферизует stdout — сломает MCP.

## D2: Имя бинарника в платформенном пакете

**Решение:** Всегда `synoema-mcp` (без версии, без платформы). На Windows — `synoema-mcp.exe`.

**Почему:** `require.resolve("@synoema/mcp-darwin-arm64/synoema-mcp")` — имя захардкожено в run.js. Версия в имени файла вынудила бы менять run.js при каждом релизе.

## D3: `files` в platform package.json

**Решение:** `"files": ["synoema-mcp", "synoema-mcp.exe"]` — npm публикует только указанные файлы.

**Почему:** Без `files` npm включит все файлы директории включая README и `.gitkeep`. Размер платформенного пакета = только бинарник (~5-10MB).

## D4: `chmod +x` в CI

**Решение:** `chmod +x npm/platforms/${PLATFORM}/synoema-mcp || true` — `|| true` чтобы не падать на Windows.

**Почему:** npm сохраняет permission bits при публикации. Если не сделать chmod перед publish, пользователь получит бинарник без execute права.

## D5: Версия в package.json — ручная или автоматическая?

**Решение:** Ручная. Перед релизом обновить все `package.json` вместе с `mcp/Cargo.toml`.

**Почему:** Автоматический bump требует jq/sed в CI и рискует сломать формат. Версия меняется редко (alpha). Можно автоматизировать позже скриптом.

## D6: `npm/platforms/*/files` — включить `.exe`?

**Решение:** `"files": ["synoema-mcp", "synoema-mcp.exe"]` — оба имени в каждом платформенном пакете.

**Почему:** npm `files` — это whitelist. Если файла нет — npm его просто не включает. Нет смысла делать разные files-массивы для каждой платформы.
