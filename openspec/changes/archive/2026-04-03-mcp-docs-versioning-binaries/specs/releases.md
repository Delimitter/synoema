# Spec: Binary Releases Directory

## Структура `releases/`

```
releases/
  README.md                    # Обзор + ссылки по платформам
  v0.1.0-alpha.1/
    README.md                  # Что в этом релизе, changelog
    darwin-arm64/
      README.md                # macOS Apple Silicon: скачать + запустить
    darwin-x64/
      README.md                # macOS Intel: скачать + запустить
    linux-x64/
      README.md                # Linux x86_64: скачать + запустить
    win32-x64/
      README.md                # Windows x64: скачать + запустить
```

## Файл: `docs/install.md`
Пошаговая инструкция для пользователей:
1. Определить платформу
2. Скачать бинарник (synoema-repl + synoema-mcp) из соответствующей директории
3. Дать права на исполнение (chmod +x, macOS gatekeeper)
4. Запустить

## CI: `.github/workflows/release.yml`
Workflow для автоматической сборки:
- Триггер: push tag `v*`
- Matrix: darwin-aarch64, darwin-x86_64, linux-x86_64, windows-x86_64
- Шаги: checkout → rust toolchain → cargo build --release → upload artifact
- Загрузка в GitHub Releases

## Бинарники в каждом релизе
- `synoema` (repl: run/jit/eval/REPL)
- `synoema-mcp` (MCP сервер)

## Соглашения по именованию
`synoema-{version}-{platform}.{ext}`
Примеры:
- `synoema-0.1.0-alpha.1-darwin-arm64` (без .exe)
- `synoema-0.1.0-alpha.1-linux-x64`
- `synoema-mcp-0.1.0-alpha.1-darwin-arm64`
- `synoema-mcp-0.1.0-alpha.1-win32-x64.exe`
