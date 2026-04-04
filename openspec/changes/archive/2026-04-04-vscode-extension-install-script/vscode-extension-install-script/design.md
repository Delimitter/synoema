# Design: VSCode Extension Install Script

## Approach
Bash-скрипт `vscode-extension/install.sh` — самодостаточный, без внешних зависимостей (кроме node/npm/code).

## Flow

```
install.sh
    │
    ├── 1. check_deps()    — node, npm, code в PATH?
    │
    ├── 2. npm install     — зависимости (esbuild, vsce, types)
    │
    ├── 3. npm run build   — esbuild → dist/extension.js
    │
    ├── 4. npx vsce package --allow-missing-repository
    │       → synoema-*.vsix
    │
    ├── 5. code --install-extension synoema-*.vsix
    │
    └── 6. cleanup         — rm node_modules, dist (optional, --keep flag)
```

## Decisions
- **npx vsce** вместо глобального vsce — не засоряем систему
- **--allow-missing-repository** — без этого vsce ругается если нет git remote
- **Цветной вывод** — через ANSI escape codes, без tput
- **set -euo pipefail** — fail fast
- **Скрипт рядом с package.json** — `vscode-extension/install.sh`
- **Cleanup опционален** — по умолчанию чистим, `--keep` оставляет артефакты

## Error handling
- Каждый шаг с `info/ok/fail` сообщениями
- Если `code` CLI нет — инструкция как поставить
- Если `node` нет — инструкция
