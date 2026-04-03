# Synoema — Полное состояние проекта

> Этот документ содержит всё необходимое для продолжения разработки проекта Synoema
> любым LLM-разработчиком. Читай этот файл ПЕРВЫМ.

---

## 0. Быстрый статус (апрель 2026)

**Версия: 0.1.0-alpha.1** — alpha-стадия, синтаксис и API могут меняться. Политика версий: [docs/versioning.md](../docs/versioning.md)

- **634 теста**, все зелёные, 0 warnings
- **Phases 9.2–18** завершены
- JIT поддерживает: int, bool, float, string, list, closures, records, record patterns, modules, ADTs, ==, !=, list comprehensions, IO (print/readline), show (все типы), type class dispatch, higher-order stdlib
- Interpreter: всё вышеперечисленное + строковый stdlib (str_slice/find/trim/...) + сетевые примитивы (tcp_listen/accept, fd_*/popen)
- Type checker: Hindley-Milner + row polymorphism + linear types (LinearArrow)
- Система диагностики: synoema-diagnostic, структурированные ошибки с span, JSON/human рендереры

---

## 1. Что такое Synoema

Synoema [sy-NO-e-ma] — язык программирования, оптимизированный для генерации кода
языковыми моделями (LLM). Название: σύν (вместе) + νόημα (содержание мысли).

**Три ключевых преимущества:**
- **-46% токенов** vs Python (12 бенчмарков, верифицировано)
- **4.4× быстрее** Python (Cranelift JIT, 3 бенчмарка)
- **100% синтаксическая корректность** через GBNF constrained decoding

## 2. Структура репозитория

```
synoema-repo/
├── synoema/                    # ← ИСХОДНЫЙ КОД КОМПИЛЯТОРА
│   ├── Cargo.toml              # Workspace manifest
│   ├── README.md               # GitHub-ready README
│   ├── LICENSE                 # MIT
│   ├── .gitignore
│   ├── crates/
│   │   ├── synoema-lexer/      # 735 строк, 82 теста — токенизация + offside rule
│   │   ├── synoema-parser/     # 1672 строки, 43 теста — Pratt parser, 15 ExprKind
│   │   ├── synoema-types/      # 1908 строк, 61 тест — Hindley-Milner inference
│   │   ├── synoema-core/       # 1536 строк, 44 теста — Core IR (System F) + optimizer
│   │   ├── synoema-eval/       # 1894 строки, 119 тестов — tree-walking interpreter
│   │   ├── synoema-codegen/    # 3044 строки, 126 тестов — Cranelift JIT + runtime
│   │   └── synoema-repl/       # 271 строка — CLI: run/jit/eval/REPL
│   ├── examples/               # 10 программ .sno
│   ├── benchmarks/
│   │   ├── token-count/        # 12 программ, BPE-бенчмарк
│   │   └── performance/        # JIT vs Python бенчмарки
│   ├── tools/
│   │   ├── bpe-verify/         # Верификация 33/33 операторов = 1 BPE-токен
│   │   └── constrained/        # synoema.gbnf + SGLang интеграция
│   ├── tests/                  # Интеграционные тесты (lexer, parser)
│   └── spec/                   # Формальная спецификация
│
├── docs/
│   ├── articles/               # 14 статей (7 RU + 7 EN) + план серии
│   ├── research/               # Научные основания (23 факта, 23 источника)
│   └── specs/                  # Language Reference, Compiler Roadmap
│
├── plans/
│   └── ROADMAP.md              # Детальный план развития
│
├── context/
│   ├── PROJECT_STATE.md        # ← ЭТОТ ФАЙЛ
│   └── DEVELOPMENT_GUIDE.md    # Инструкции для LLM-разработчика
│
└── CLAUDE.md                   # Быстрая справка для Claude Code
```

## 3. Текущие метрики

| Метрика | Значение |
|---------|----------|
| Строк Rust | ~12000 |
| Тестов | 634 (все зелёные) |
| Warnings | 0 |
| Крейтов | 8 (добавлен synoema-diagnostic) |
| Примеров | 14 программ (.sno) |
| BPE-aligned операторов | 33/33 |
| Экономия токенов vs Python | 46% |
| Ускорение vs Python (JIT) | 4.4× среднее |
| GBNF-грамматика | 162 строки, 48 правил |

## 4. Что работает

### Interpreter (`synoema run`) — ВСЕ фичи:
- Pattern matching (single-arg, multi-arg, nested, cons)
- Closures, higher-order functions (map, filter, fold)
- Списки: `[1 2 3]`, cons `x:xs`, concat `++`, comprehensions `[x | x <- xs, p x]`
- Строки: `"hello"`, конкатенация `++`, `show`
- Условия: `? cond -> then : else`
- Where-блоки: `f x = y + z` / `  y = ...` / `  z = ...`
- Let-polymorphism: `id` как `Int → Int` и `Bool → Bool` в одной программе
- Pipe: `x |> f |> g`
- Рекурсия: factorial, fibonacci, quicksort, ackermann
- **Records (Phase 9.4):** `{x = 3, y = 4}`, field access `p.x`, pattern matching
- **Modules (Phase 9.5):** `mod Math`, `use Math (square pi)` — lexical namespacing
- **Row polymorphism (Phase 11.2):** `get_x r = r.x` принимает `{x=3, y=4}` и `{x=1, z=true}` — Rémy-style row unification

### JIT (`synoema jit`) — числа + списки + closures + строки + records + компрехеншны:
- Целочисленная арифметика: +, -, *, /, %
- Сравнения: ==, !=, <, >, <=, >=  (universal: работает для int и string)
- Логические: &&, ||, !
- Pattern matching (литералы, переменные, Nil, Cons)
- Рекурсия (включая multi-equation: gcd, pow, collatz)
- Списки: MkList, cons, concat, head, tail, length, sum
- **Closures (Phase 9.2):** lambda lifting, indirect calls, higher-order functions (map, filter)
- **Строки (Phase 9.3):** tagged pointer scheme (bit 1), StrNode, show/++/length/== на строках, fizzbuzz
- **List comprehensions:** `[x*x | x <- xs]`, `[x | x <- xs, x > 3]` — через synoema_concatmap FFI
- **Records (Phase 9.4):** `{x = 3, y = 4}`, `r.field` — RecordNode heap alloc, FNV-hash field lookup
- **String equality:** `"hello" == "hello"` → true — `synoema_val_eq` runtime dispatch
- **Constant folding (Phase 10.2):** `2 + 3 → 5`, `? true -> x : y → x` на этапе компиляции
- **ADTs (Phase 11.1):** `Maybe a = Just a | None`, multi-equation matching, ConNode heap alloc, tag comparison
- **Nested ADT patterns (Phase 11.3):** `Just (MkPair x y)` — вложенные конструкторы в JIT, 2 теста
- **Full ADT matching (Phase 11.4):** `Just 0` literal sub-patterns, тройная вложенность `Just (Just (Just x))`, рекурсивный `bind_sub_pat`, 4 теста
- **String literal patterns (Phase 11.5):** `greet "Alice" = "Hello"`, строковые суб-паттерны внутри конструкторов, 5 тестов
- **Float (Phase 12a):** `3.14`, арифметика `+ - * /`, сравнения, условия со строками — FloatNode heap-alloc, tag=0x04, 10 тестов
- **Record patterns (Phase 12b):** `get_x {x = v, y = _} = v` — `CorePat::Record` в JIT через `synoema_record_get` + FNV-хэш, 5 тестов
- show возвращает строковое значение (tagged i64 ptr), compile_and_display для human-readable вывода
- Heap-allocated linked list + string + record + ConNode + FloatNode runtime

### JIT НЕ поддерживает:
- Effects / IO monad
- Type class dispatch

## 5. Верифицированные результаты

### JIT vs Interpreter (должны совпадать):
```
factorial.sno  JIT=3628800   Interp=3628800   ✓
gcd.sno        JIT=21        Interp=21         ✓
pow.sno        JIT=1048576   Interp=1048576    ✓
collatz.sno    JIT=111       Interp=111        ✓
euler1.sno     JIT=233168    Interp=233168     ✓ (Phase 10.1 TCO: 64MB stack thread)
fizzbuzz.sno   JIT=FizzBuzz  Interp=FizzBuzz   ✓ (Phase 9.3 strings in JIT)
```

### Performance (JIT vs CPython 3.12):
```
fib(30):      5.9× faster
gcd(100K):    1.7× faster
collatz(10K): 5.6× faster
Average:      4.4× faster
```

## 6. Известные баги

0 известных багов, 475/488 тестов зелёные.

### Исправленные баги:
| Баг | Решение |
|-----|---------|
| Float арифметика (+,-,*,/,**) сломана в `synoema run` и `jit` | Type checker: is_float detection для Num-полиморфизма (infer.rs) |
| String concat (++) не проходил type check | Type checker: is_string detection для ++ (infer.rs) |
| Division by zero для смешанных типов (Int/Float) | Единый is_zero check для всех числовых комбинаций (eval.rs) |
| Integer power overflow (10**20 паника) | checked_pow + try_from для overflow-safe ** (eval.rs) |
| Float powi i64→i32 truncation | try_from с fallback на powf (eval.rs) |
| Optimizer не фолдил Pow, FPow, float comparisons | Добавлены все missing cases в fold_binary/fold_unary (optimize.rs) |
| Euler1 stack overflow в interpreter | Phase 10.1: iterative TCO loop + 64MB stack thread |
| closure_filter_length crash | Неправильный синтаксис в тесте: cons `:` конфликтовал с ternary `:`. Исправлено: явные скобки |
| JIT не поддерживал строки | Phase 9.3: tagged pointer scheme (bit 1), StrNode, show/++/length |
| «Ackermann JIT bug» (false positive) | Баг не существовал: `ack 3 4 = 125` — правильный ответ (2^7 − 3) |

## 7. Архитектура компилятора

```
Source (.sno)
  │
  ▼
LEXER (synoema-lexer)
  Offside rule: отступы → INDENT/DEDENT токены
  33 оператора, каждый — 1 BPE-токен
  │
  ▼
PARSER (synoema-parser)
  Pratt parser с 13 уровнями приоритета
  15 ExprKind: Lit, Var, App, Lam, Let, Cond, BinOp, UnOp,
               List, ListComp, Cons, Case, Pipe, Compose, Where
  6 Pattern: Var, Lit, Wildcard, Cons, Con, Paren
  │
  ▼
TYPE CHECKER (synoema-types)
  Hindley-Milner (Algorithm W)
  Let-polymorphism с correct generalization
  Встроенные типы: Int, Bool, String, List a, a → b
  │
  ▼
CORE IR (synoema-core)
  Десахаризация → System F:
  - Pattern match → case expressions
  - List comprehension → concatMap
  - Where → nested let
  - Multi-equation → equation chain с guards
  - Pipe/compose → function application
  │
  ├──────────────────────┐
  ▼                      ▼
INTERPRETER            CRANELIFT JIT
(synoema-eval)         (synoema-codegen)
  Tree-walking           compiler.rs: Core IR → Cranelift IR → x86-64
  Closures: Env capture  runtime.rs: heap-allocated linked lists
  Lists: Cons/Nil ADT      FFI: synoema_nil, synoema_cons, synoema_head,
  Strings: Rust String           synoema_tail, synoema_concat, synoema_length,
  Full pattern matching          synoema_sum, synoema_print_int, synoema_print_list
```

## 8. Ключевые файлы (что менять для каких задач)

| Задача | Файл(ы) |
|--------|---------|
| Новый оператор/синтаксис | lexer/src/lexer.rs → parser/src/parser.rs |
| Новый тип данных | types/src/types.rs + types/src/infer.rs |
| Новая десахаризация | core/src/desugar.rs |
| Новый PrimOp в JIT | codegen/src/compiler.rs (compile_binop/compile_unop) |
| Новый runtime FFI | codegen/src/runtime.rs + регистрация в compiler.rs (new() + declare_runtime_functions()) |
| Новый pattern в JIT | codegen/src/compiler.rs (compile_case) |
| Новая CLI команда | repl/src/main.rs |
| Новый пример | examples/*.sno |
| Обновить грамматику | tools/constrained/synoema.gbnf |

## 9. Как запустить

```bash
cd synoema/
cargo build                    # Сборка
cargo test                     # 264 теста
cargo run -p synoema-repl -- run examples/quicksort.sno   # Interpreter
cargo run -p synoema-repl -- jit examples/factorial.sno    # JIT
cargo run -p synoema-repl -- eval "6 * 7"                  # Eval
cargo run -p synoema-repl                                  # REPL
cargo build --release -p synoema-repl                       # Release build для бенчмарков
```

## 10. Зависимости

```toml
# Workspace Cargo.toml
[workspace.dependencies]
cranelift-codegen = "0.113"
cranelift-frontend = "0.113"
cranelift-jit = "0.113"
cranelift-module = "0.113"
cranelift-native = "0.113"
pretty_assertions = "1"
```

Минимальные зависимости: только Cranelift + pretty_assertions для тестов.
Нет tokio, нет serde, нет async — чистый синхронный Rust.
