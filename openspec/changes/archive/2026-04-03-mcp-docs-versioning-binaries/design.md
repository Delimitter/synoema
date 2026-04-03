# Design: MCP Docs, Versioning, Binary Releases

## D1: Размещение документации

**Решение:** `docs/mcp.md` и `docs/versioning.md` и `docs/install.md`.

**Альтернативы:**
- `mcp/README.md` — слишком узкий scope, не видно из корня репо
- Прямо в README.md — README уже большой (300+ строк)

**Обоснование:** Паттерн уже используется в проекте (`docs/testing.md`, `docs/stress-server.md`). README.md содержит краткое описание + ссылки на docs/*.

## D2: Формат версии

**Решение:** `0.1.0-alpha.1` (SemVer + pre-release identifier).

**Альтернативы:**
- `0.1.0-alpha` без номера: нельзя различить alpha-релизы
- `0.1.0.alpha1` без дефиса: нарушает SemVer
- `v0.1.0-alpha.1` с `v`: для git tags, не для Cargo.toml

**Обоснование:** SemVer 2.0 стандарт. Cargo поддерживает pre-release версии. N позволяет делать несколько alpha-релизов с одним MAJOR.MINOR.PATCH.

## D3: Структура releases/ — README-файлы vs реальные бинарники

**Решение:** README-файлы в директориях + CI workflow. Реальные бинарники — через GitHub Releases (не коммитятся в репо).

**Альтернативы:**
- Коммитить бинарники в git: раздует репо, antipattern
- Только CI, без releases/ структуры: пользователь не знает куда идти

**Обоснование:** GitHub best practice — бинарники в Releases, исходники в репо. Структура releases/ служит документацией и шаблоном для CI, но сами бинарники не трекаются git-ом.

## D4: Число поддерживаемых платформ

**Решение:** darwin-arm64, darwin-x64, linux-x64, win32-x64.

**Исключены:** linux-arm64, linux-musl, freebsd — можно добавить позже.

**Обоснование:** Покрывает >95% разработчиков. Cranelift поддерживает все 4 платформы.

## D5: Обновление Cargo.toml версий

**Решение:** Обновить версию в `mcp/Cargo.toml` и `mcp/synoema-mcp/Cargo.toml` до `0.1.0-alpha.1`. Версию в `lang/` не трогать (отдельный workspace, отдельный цикл релизов).

**Обоснование:** MCP — отдельный crate с собственным release cycle. Lang/compiler версия пока остаётся `0.1.0` до выхода из alpha.
