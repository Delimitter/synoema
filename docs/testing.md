# Тестирование

Synoema покрыт 771 тестом в 7 крейтах. Все тесты проходят с `0 failures, 0 warnings`.

## Быстрый старт

```bash
cd lang/
cargo test        # все тесты — 771/771 зелёных
```

## Структура тестов

### Unit-тесты

Каждый крейт содержит интеграционные тесты в `src/tests.rs`:

| Крейт | Тестов | Что тестируется |
|-------|-------:|-----------------|
| `synoema-lexer` | 97 | Токенизация, offside rule, escape-последовательности, string interpolation |
| `synoema-parser` | 72 | Pratt-парсер, type aliases, imports, error recovery, string interp |
| `synoema-types` | 90 | Hindley-Milner, row polymorphism, type classes, alias expansion |
| `synoema-core` | 50 | Core IR, десахаризация, оптимизации |
| `synoema-eval` | 183 | Tree-walking интерпретатор, все фичи языка |
| `synoema-codegen` | 209 | Cranelift JIT — арифметика, строки, ADT, замыкания |
| `synoema-diagnostic` | — | Нет отдельных тестов (покрыто через eval/codegen) |
| **Итого** | **771** | |

### Стресс-тесты

Стресс-тесты находятся в `tests/stress.rs` каждого крейта и проверяют производительность и стабильность при больших нагрузках:

```bash
# Запустить стресс-тесты конкретного крейта
cargo test --test stress -p synoema-lexer   -- --nocapture
cargo test --test stress -p synoema-types   -- --nocapture
cargo test --test stress -p synoema-eval    -- --nocapture
cargo test --test stress -p synoema-codegen -- --nocapture
```

| Крейт | Стресс-тестов | Примеры |
|-------|--------------|---------|
| `synoema-lexer` | 10 (+ 3 игнор.) | 100K токенов, глубокая вложенность |
| `synoema-types` | 9 (+ 2 игнор.) | 500 функций, 100 ADT-вариантов |
| `synoema-eval` | 17 (+ 6 игнор.) | fib(25), сортировка 10K, typeclass dispatch |
| `synoema-codegen` | 23 (+ 9 игнор.) | fib(35) via JIT, 1K итераций map/filter |

Тесты с `#[ignore]` требуют флага `--ignored` и могут занимать длительное время:

```bash
cargo test --test stress -p synoema-eval -- --ignored --nocapture
```

## Встроенные тесты языка

Synoema поддерживает три вида тестов в `.sno` файлах:

### Doctests
```
--- example: fact 5 == 120
fact n = ? n == 0 -> 1 : n * fact (n - 1)
```

### Test declarations
```
test "fact base" = fact 0 == 1
test "sort then reverse" = reverse (qsort [3 1 2]) == [3 2 1]
```

### Property-based tests
```
test "reverse involution" = prop xs -> reverse (reverse xs) == xs
test "fact positive" = prop n -> fact n > 0 when n >= 0 && n <= 10
```

### Запуск

```bash
cargo run -p synoema-repl -- test examples/testing.sno          # один файл
cargo run -p synoema-repl -- test examples/                     # директория
cargo run -p synoema-repl -- test examples/ --filter "sort"     # фильтр по имени
```

Keywords: `test` (декларация), `prop` (property-генератор), `when` (условный property).
Все три — 1 BPE-токен в cl100k_base.

## Запуск по крейтам

```bash
# Только один крейт
cargo test -p synoema-lexer
cargo test -p synoema-parser
cargo test -p synoema-types
cargo test -p synoema-core
cargo test -p synoema-eval
cargo test -p synoema-codegen

# С выводом (не глотать println!)
cargo test -p synoema-eval -- --nocapture

# Конкретный тест
cargo test -p synoema-eval -- test_factorial

# Параллельно в один поток (для детерминированного вывода)
cargo test -p synoema-eval -- --test-threads=1
```

## Интерактивный дашборд

Для визуального запуска тестов в браузере используйте сервер на Synoema:

```bash
cd lang/
cargo run -p synoema-repl -- run examples/stress_server.sno
# Откройте: http://localhost:8765/stress_tests.html
```

Дашборд показывает результаты в реальном времени через SSE-стриминг. Подробнее: [docs/stress-server.md](stress-server.md).

## Производительность (release vs debug)

Тесты по умолчанию запускаются в debug-режиме, который ~10× медленнее release:

```bash
# Debug (по умолчанию)
cargo test

# Release
cargo test --release
```

Стресс-тесты с жёсткими временными ограничениями помечены `#[cfg(not(debug_assertions))]` и пропускаются в debug-сборке.

## Правила для новых тестов

- `cargo test` должен быть чистым (0 failures, 0 warnings) перед каждым коммитом
- Новые фичи: сначала тест в interpreter, потом в JIT
- Стресс-тесты, которые переполняют стек в debug (> ~1500 уровней рекурсии), помечаются `#[ignore]`
- Тесты с допущениями о производительности оборачиваются в `#[cfg(not(debug_assertions))]`

## CI

```bash
# Команда для CI (эквивалентна cargo test)
cargo test 2>&1 | grep -E "test result|FAILED"
```

Ожидаемый вывод — строки вида `test result: ok. N passed; 0 failed`.
