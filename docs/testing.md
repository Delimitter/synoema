# Тестирование

Synoema покрыт 634 тестами в 7 крейтах. Все тесты проходят с `0 failures, 0 warnings`.

## Быстрый старт

```bash
cd lang/
cargo test        # все тесты — 634/634 зелёных
```

## Структура тестов

### Unit-тесты

Каждый крейт содержит интеграционные тесты в `src/tests.rs`:

| Крейт | Тестов | Что тестируется |
|-------|-------:|-----------------|
| `synoema-lexer` | 51 | Токенизация, offside rule, escape-последовательности |
| `synoema-parser` | 43 | Pratt-парсер, 15 видов ExprKind, ошибки |
| `synoema-types` | 61 | Hindley-Milner, row polymorphism, type classes |
| `synoema-core` | 44 | Core IR, десахаризация, оптимизации |
| `synoema-eval` | 137 | Tree-walking интерпретатор, все фичи языка |
| `synoema-codegen` | 191 | Cranelift JIT — арифметика, строки, ADT, замыкания |
| `synoema-diagnostic` | — | Нет отдельных тестов (покрыто через eval/codegen) |
| **Итого** | **634** | |

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
