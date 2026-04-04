# Tasks: npm-publication

## Checklist

- [x] 1. Синхронизировать версии: все 5 package.json → `0.1.0-alpha.1`, optionalDependencies → `0.1.0-alpha.1`
- [x] 2. Создать `scripts/npm-bump-version.sh` — атомарный bump всех npm-пакетов
- [x] 3. Обновить `.github/workflows/release.yml` — инъекция версии из git tag перед каждым `npm publish`
- [x] 4. Обновить `docs/versioning.md` — секция "NPM Release Process" с чеклистом
- [x] 5. Обновить `releases/README.md` — npm install инструкции
- [x] 6. Обновить `context/PROJECT_STATE.md` — npm distribution в быстром статусе

## Apply Requires

- proposal.md
- specs/npm-versioning.md
- design.md
