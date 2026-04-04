# Delta Spec: npm-versioning

## Capability

Все npm-пакеты проекта версионируются синхронно из единого источника (git tag). Версия инжектируется автоматически при CI/CD публикации.

## Current State

- `npm/synoema-mcp/package.json`: `0.1.0-alpha.4`
- `npm/platforms/*/package.json`: `0.1.0-alpha.1`
- CI: статичные версии, нет инъекции из tag

## Target State

- Все 5 package.json: единая версия `0.1.0-alpha.1` (baseline)
- `scripts/npm-bump-version.sh`: обновляет версию во всех package.json + optionalDependencies
- CI workflow: step `Set npm package versions` инжектирует версию из git tag перед publish
- versioning.md: секция "NPM Release Process" с чеклистом

## Invariants

- Version в package.json ≤ version в git tag (CI перезаписывает)
- optionalDependencies в main package всегда ссылаются на ту же версию что main
- npm tag: `alpha` для alpha-стадии, `beta` для beta, `latest` для stable
