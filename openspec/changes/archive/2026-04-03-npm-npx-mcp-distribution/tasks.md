# Tasks: npm/npx Distribution

## Checklist

### A. Структура директорий

- [ ] A1: Создать `npm/synoema-mcp/bin/` и `npm/platforms/{darwin-arm64,darwin-x64,linux-x64,win32-x64}/`

### B. package.json файлы

- [ ] B1: Создать `npm/synoema-mcp/package.json` — главный пакет
- [ ] B2: Создать `npm/platforms/darwin-arm64/package.json`
- [ ] B3: Создать `npm/platforms/darwin-x64/package.json`
- [ ] B4: Создать `npm/platforms/linux-x64/package.json`
- [ ] B5: Создать `npm/platforms/win32-x64/package.json`
- [ ] B6: Создать `.gitkeep` в каждой `npm/platforms/*/` директории (placeholder для бинарников)

### C. JS-обёртка

- [ ] C1: Создать `npm/synoema-mcp/bin/run.js` — platform detection + spawnSync

### D. CI workflow

- [ ] D1: Обновить `.github/workflows/release.yml` — добавить npm publish шаги в build job
- [ ] D2: Добавить job `publish-main` в `release.yml`

### E. Документация

- [ ] E1: Обновить `docs/mcp.md` — добавить раздел "Install via npx/npm"
- [ ] E2: Обновить `docs/install.md` — добавить Option 0 (npx, самый простой)
- [ ] E3: Обновить `README.md` — упомянуть `npx synoema-mcp` в MCP разделе

---

## Детализация

### C1: run.js

```js
#!/usr/bin/env node
const { spawnSync } = require("child_process");
const { platform, arch } = process;

const PKGS = {
  "darwin arm64": "@synoema/mcp-darwin-arm64",
  "darwin x64":   "@synoema/mcp-darwin-x64",
  "linux x64":    "@synoema/mcp-linux-x64",
  "win32 x64":    "@synoema/mcp-win32-x64",
};

const pkg = PKGS[`${platform} ${arch}`];
if (!pkg) {
  process.stderr.write(`synoema-mcp: unsupported platform ${platform}/${arch}\n`);
  process.stderr.write(`Build from source: https://github.com/Delimitter/synoema\n`);
  process.exit(1);
}

let bin;
try {
  bin = require.resolve(`${pkg}/synoema-mcp`);
} catch {
  process.stderr.write(`synoema-mcp: platform package ${pkg} not found\n`);
  process.stderr.write(`Try: npm install ${pkg}\n`);
  process.exit(1);
}

const { status, error } = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
if (error) { process.stderr.write(`${error}\n`); process.exit(1); }
process.exit(status ?? 0);
```

### D1: Изменения в release.yml (build job)

После существующего шага "Rename binaries" добавить:
1. Copy binary to npm/platforms dir
2. setup-node@v4 с registry-url
3. npm publish для платформенного пакета

### D2: Новый job publish-main

needs: [build] (все 4 платформы)
steps: checkout → setup-node → npm publish в npm/synoema-mcp/
