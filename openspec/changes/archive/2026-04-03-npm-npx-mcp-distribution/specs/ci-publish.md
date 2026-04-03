# Spec: CI npm Publishing

## Изменения в `.github/workflows/release.yml`

### В существующий build job — добавить шаг после rename

```yaml
- name: Copy binary to npm platform dir
  shell: bash
  run: |
    VERSION=${{ steps.version.outputs.VERSION }}
    PLATFORM=${{ matrix.platform.name }}
    EXT=${{ matrix.platform.ext }}
    cp synoema-mcp-${VERSION}-${PLATFORM}${EXT} \
       npm/platforms/${PLATFORM}/synoema-mcp${EXT}
    chmod +x npm/platforms/${PLATFORM}/synoema-mcp${EXT} || true

- name: Setup Node.js
  uses: actions/setup-node@v4
  with:
    node-version: '20'
    registry-url: 'https://registry.npmjs.org'

- name: Publish platform package
  working-directory: npm/platforms/${{ matrix.platform.name }}
  run: npm publish --access public
  env:
    NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

### Новый job `publish-main` (после `build`)

```yaml
publish-main:
  name: Publish main npm package
  needs: build
  runs-on: ubuntu-22.04
  steps:
    - uses: actions/checkout@v4
    - uses: actions/setup-node@v4
      with:
        node-version: '20'
        registry-url: 'https://registry.npmjs.org'
    - name: Publish synoema-mcp
      working-directory: npm/synoema-mcp
      run: npm publish --access public
      env:
        NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

## Требования

- GitHub Secret `NPM_TOKEN` — automation token с publish правами для org `@synoema` и пакета `synoema-mcp`
- npm org `@synoema` должна существовать (одноразово через npmjs.com)
- Версии в `npm/*/package.json` должны совпадать с git тегом

## Порядок публикации

1. Сначала 4 платформенных пакета (параллельно в matrix)
2. Затем главный `synoema-mcp` (после `needs: build`)

Это важно: npm при установке `synoema-mcp` сразу резолвит `optionalDependencies` — они должны существовать к моменту публикации главного пакета.
