# Spec: all_crates() Cache

## Capability

Кэширование результата `all_crates()` с инвалидацией по mtime директории `lang/crates/`.

## Current Behavior

- `all_crates()` при каждом вызове делает `read_dir` + `get_crate()` для каждого подкаталога
- Файлы внутри крейтов кэшируются по mtime, но сам список крейтов — нет

## New Behavior

- `LiveIndex` хранит кэш `all_crates` результата: `(SystemTime, Vec<CrateSummary>)`
- При вызове проверяется mtime директории `lang/crates/`
- Если mtime не изменился — возвращается кэш
- Если изменился — полный пересчёт

## Changes

### index.rs

1. Добавить поле `crates_cache: Mutex<Option<(SystemTime, Vec<CrateSummary>)>>` в `LiveIndex`
2. В `all_crates()` проверять mtime `lang/crates/` → при совпадении возвращать кэш
3. Обновить `INDEX` static инициализацию

## Acceptance Criteria

- Повторный вызов `all_crates()` без изменений в директории использует кэш
- Существующие тесты проходят
