# Synoema — Полное состояние проекта

> Этот документ содержит всё необходимое для продолжения разработки проекта Synoema
> любым LLM-разработчиком. Читай этот файл ПЕРВЫМ.

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
│   │   ├── synoema-lexer/      # 706 строк, 80 тестов — токенизация + offside rule
│   │   ├── synoema-parser/     # 1398 строк, 36 тестов — Pratt parser, 15 ExprKind
│   │   ├── synoema-types/      # 1453 строки, 42 теста — Hindley-Milner inference
│   │   ├── synoema-core/       # 969 строк, 26 тестов — Core IR (System F)
│   │   ├── synoema-eval/       # 1314 строк, 46 тестов — tree-walking interpreter
│   │   ├── synoema-codegen/    # 944 строки, 34 теста — Cranelift JIT + runtime
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
| Строк Rust | 7055 |
| Тестов | 264 (все зелёные) |
| Warnings | 1 (unused `fresh` в desugar.rs) |
| Примеров | 10 программ (.sno) |
| BPE-aligned операторов | 33/33 |
| Экономия токенов vs Python | 46% |
| Ускорение vs Python (JIT) | 4.4× среднее |
| GBNF-грамматика | 145 строк, 41 правило |
| Статей написано | 14 (7 RU + 7 EN) |

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

### JIT (`synoema jit`) — числа + списки:
- Целочисленная арифметика: +, -, *, /, %
- Сравнения: ==, !=, <, >, <=, >=
- Логические: &&, ||, !
- Pattern matching (литералы, переменные, Nil, Cons)
- Рекурсия (включая multi-equation: gcd, pow, collatz)
- Списки: MkList, cons, concat, head, tail, length, sum
- show/print через FFI
- Heap-allocated linked list runtime

### JIT НЕ поддерживает (нужна Phase 9.2+):
- Closures как значения (map f xs — f как переменная)
- Строки в JIT
- List comprehensions через JIT (desugaring в concatMap требует closures)
- Records
- Модули

## 5. Верифицированные результаты

### JIT vs Interpreter (должны совпадать):
```
factorial.sno  JIT=3628800   Interp=3628800   ✓
gcd.sno        JIT=21        Interp=21         ✓
pow.sno        JIT=1048576   Interp=1048576    ✓
collatz.sno    JIT=111       Interp=111        ✓
euler1.sno     JIT=233168    Interp=overflow   ✓ (JIT only — interp needs TCO)
```

### Performance (JIT vs CPython 3.12):
```
fib(30):      5.9× faster
gcd(100K):    1.7× faster
collatz(10K): 5.6× faster
Average:      4.4× faster
```

## 6. Известные баги

| Баг | Причина | Как воспроизвести | Решение |
|-----|---------|-------------------|---------|
| Ackermann JIT ≠ Interpreter | 3-equation multi-arg pattern match в desugar | `synoema jit examples/ackermann.sno` → 125 (должно 5) | Доработать build_equation_chain для 3+ equations с литералами в разных позициях |
| Euler1 stack overflow (interp) | Рекурсия 999 уровней без TCO | `synoema run examples/euler1.sno` | Phase 10.1: tail call optimization |
| 1 warning | Unused `fresh` в build_pattern_guard | `cargo build` | Переименовать в `_fresh` только в этой функции |

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
