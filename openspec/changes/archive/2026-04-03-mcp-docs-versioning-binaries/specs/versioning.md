# Spec: Versioning Policy

## Текущая версия
`0.1.0-alpha.1`

## Схема версионирования
`MAJOR.MINOR.PATCH-STAGE.N`

| Компонент | Значение |
|-----------|---------|
| MAJOR | Breaking changes в языке или core API |
| MINOR | Новые возможности (обратно совместимые) |
| PATCH | Исправления ошибок |
| STAGE | `alpha` / `beta` / (пусто = stable) |
| N | Инкрементальный номер внутри stage |

## Стадии

### Alpha (`0.x.y-alpha.N`)
- Синтаксис языка может меняться
- ABI JIT не гарантирован между версиями
- MCP API может меняться
- Рекомендуется только для исследователей и ранних adopters

### Beta (`0.x.y-beta.N`)
- Синтаксис языка зафиксирован
- MCP API стабилен
- ABI JIT стабилен в рамках MINOR
- Допустимо для прототипов и экспериментов

### Stable (`1.x.y`)
- Полная обратная совместимость в рамках MAJOR
- Семантическое версионирование (SemVer)
- Гарантии: синтаксис, stdlib API, MCP tools/resources API

## Файл: `docs/versioning.md`
Полный документ с описанием политики для пользователей.

## Изменения в других документах
- `README.md`: badge с текущей версией и стадией
- `context/PROJECT_STATE.md`: раздел о текущей версии и стадии
- `mcp/synoema-mcp/Cargo.toml`: version = "0.1.0-alpha.1"
- `mcp/Cargo.toml`: version = "0.1.0-alpha.1"
