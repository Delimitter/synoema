# Proposal: npm-publication

## Problem

NPM-пакеты (`synoema-mcp`, `@delimitter/mcp-*`) никогда не были реально опубликованы на npm registry. При этом:

1. **Рассинхрон версий** — `synoema-mcp` на `0.1.0-alpha.4`, платформенные пакеты на `0.1.0-alpha.1`. При публикации main-пакет не найдёт свои optional dependencies
2. **Нет механизма синхронного version bump** — версии в 5 package.json обновляются вручную, что гарантирует ошибки
3. **CI workflow не инжектирует версию из git tag** — версии в package.json статичные, а не из tag
4. **Документация не описывает release flow** — versioning.md не покрывает npm-специфику, нет чеклиста релиза

## Scope

1. **Синхронизация версий** — все 5 package.json на одну версию `0.1.0-alpha.1`
2. **Скрипт version bump** — `scripts/npm-bump-version.sh` для атомарного обновления всех пакетов
3. **CI workflow fix** — инжекция версии из git tag в package.json перед `npm publish`
4. **Документация** — обновить versioning.md (npm release process), releases/README.md, PROJECT_STATE.md

## Out of Scope

- Реальная публикация на npm (требует NPM_TOKEN)
- Добавление новых платформ
- Изменение архитектуры пакетов (launcher pattern остаётся)
- Изменение имён пакетов или org scope

## Success Criteria

- Все 5 package.json на единой версии
- `scripts/npm-bump-version.sh` обновляет все пакеты за одну команду
- CI workflow инжектирует версию из git tag
- versioning.md документирует npm release flow
- `cargo test` чистый (0 failures, 0 warnings)
