# Proposal: VSCode Extension Install Script

## Problem
Установка VSCode-расширения Synoema требует нескольких ручных шагов: `npm install`, `npm run build`, `npm run package`, `code --install-extension`. Нужен один скрипт, который делает всё.

## Solution
Создать `vscode-extension/install.sh` — один скрипт, который:
1. Проверяет зависимости (node, npm, code CLI)
2. Устанавливает npm-зависимости
3. Собирает расширение (esbuild)
4. Пакует в .vsix (vsce)
5. Устанавливает в VSCode
6. Чистит артефакты сборки (опционально)

## Scope
- Один файл: `vscode-extension/install.sh`
- Поддержка macOS и Linux
- Цветной вывод с понятными сообщениями об ошибках
- Идемпотентный — можно запускать повторно

## Non-goals
- Windows-скрипт (пока нет)
- Публикация в Marketplace
- Сборка CLI (предполагается, что уже есть или пользователь поставит отдельно)
