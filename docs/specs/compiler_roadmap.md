# Synoema: План реализации компилятора

## От спецификации к работающему компилятору

**Текущее состояние:** Готовы 3 документа
- ✅ Научные основания (верифицированные факты, 23 источника)
- ✅ Обзор экосистемы и ниши (GitHub-разведка)
- ✅ Language Reference v0.1 (лексика, грамматика, типы, семантика)

**Целевое состояние:** Компилятор Synoema → LLVM IR → native binary,
с интеграцией constrained decoding для LLM-генерации кода.

---

## Общая архитектура (что мы строим)

```
┌─────────────────────────────────────────────────────────┐
│                    Synoema Toolchain                       │
│                                                         │
│  ┌──────────┐  ┌──────────┐  ┌────────┐  ┌──────────┐  │
│  │  Lexer   │→ │  Parser  │→ │  Type  │→ │ Core IR  │  │
│  │(offside) │  │ (Pratt)  │  │ Check  │  │(System F)│  │
│  └──────────┘  └──────────┘  └────────┘  └────┬─────┘  │
│                                                │        │
│                              ┌─────────────────┤        │
│                              ▼                 ▼        │
│                     ┌──────────────┐  ┌──────────────┐  │
│                     │ Interpreter  │  │  LLVM Code   │  │
│                     │   (REPL)     │  │  Generator   │  │
│                     └──────────────┘  └──────┬───────┘  │
│                                              │          │
│                                              ▼          │
│                                     ┌──────────────┐    │
│                                     │ Native / WASM│    │
│                                     └──────────────┘    │
│                                                         │
│  ┌──────────────────────────────────────────────────┐   │
│  │         Constrained Decoding Module               │   │
│  │  DCFG → XGrammar FSM → Token Mask Cache           │   │
│  │  Type Environment → Semantic Constraints           │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

---

## ФАЗА 0: Подготовка инфраструктуры
**Срок: 1 неделя**

### 0.1 Репозиторий и структура проекта
```
synoema/
├── crates/
│   ├── synoema-lexer/        -- лексический анализатор
│   ├── synoema-parser/       -- синтаксический анализатор
│   ├── synoema-types/        -- система типов, Algorithm W
│   ├── synoema-core/         -- Core IR (десахаризация)
│   ├── synoema-eval/         -- tree-walking интерпретатор
│   ├── synoema-codegen/      -- LLVM IR генератор (Фаза 3)
│   └── synoema-repl/         -- REPL
├── spec/
│   ├── grammar.ebnf        -- формальная грамматика
│   ├── typing-rules.tex    -- правила типизации
│   ├── bpe-alignment.json  -- маппинг операторов на BPE-токены
│   └── examples/           -- каноничные примеры программ
├── tests/
│   ├── lexer/
│   ├── parser/
│   ├── typechecker/
│   ├── eval/
│   └── integration/
├── benchmarks/
│   ├── token-count/        -- сравнение токенов Synoema vs Python
│   └── performance/        -- runtime benchmarks (Фаза 3)
├── tools/
│   ├── bpe-verify/         -- скрипт проверки BPE-alignment
│   └── constrained/        -- модуль для XGrammar (Фаза 4)
├── Cargo.toml
├── README.md
└── LICENSE                  -- MIT или Apache 2.0
```

**Задачи:**
- [ ] Создать GitHub-репозиторий
- [ ] Настроить Rust workspace (Cargo.toml с crates)
- [ ] CI/CD: GitHub Actions (cargo test, cargo clippy, cargo fmt)
- [ ] Перенести grammar.ebnf из Language Reference
- [ ] Создать bpe-alignment.json с верифицированными данными
- [ ] README с описанием проекта и ссылками на научные основания

**Критерий готовности:** `cargo build` проходит, CI зелёный, README содержит pitch проекта.

---

## ФАЗА 1: Лексер
**Срок: 2 недели**

### 1.1 Базовый лексер (неделя 1)

Преобразование исходного текста в поток токенов.

**Задачи:**
- [ ] Определить enum Token со всеми вариантами:
  ```rust
  enum Token {
      // Литералы
      Int(i64), Float(f64), Str(String), Char(char), Bool(bool),
      // Идентификаторы
      LowerId(String), UpperId(String),
      // Ключевые слова
      Mod, Use, Trait, Impl, True, False,
      // Операторы (каждый — отдельный вариант)
      Arrow,      // ->
      BackArrow,  // <-
      Pipe,       // |>
      Concat,     // ++
      Eq, Neq, Lt, Gt, Lte, Gte,
      And, Or,
      Compose,    // >>
      Plus, Minus, Star, Slash, Percent,
      Dot, Colon, Assign, At, Bar, Underscore, Comma, DotDot,
      Question,   // ?
      Lambda,     // backslash
      // Разделители
      LParen, RParen, LBracket, RBracket,
      // Структурные
      Indent, Dedent, Newline,
      // Специальные
      Comment(String), EOF,
  }
  ```
- [ ] Лексер: итератор `&str → Vec<Token>`
- [ ] Обработка: числа, строки (escape-последовательности), идентификаторы
- [ ] Различение ключевых слов и идентификаторов (lookup table)
- [ ] Позиционная информация: `Span { line, col, offset }` для каждого токена

**Тесты (не менее 30):**
- [ ] Каждый оператор распознаётся корректно
- [ ] Строки с escape-последовательностями
- [ ] Числа: целые, с плавающей точкой, отрицательные
- [ ] Ключевые слова vs идентификаторы
- [ ] Edge cases: пустой ввод, только комментарии, unicode в строках

### 1.2 Offside rule / индентация (неделя 2)

Генерация INDENT/DEDENT токенов из отступов (как в Python, но проще).

**Алгоритм:**
```
indent_stack = [0]
для каждой строки:
  level = количество начальных пробелов / 2
  если level > top(indent_stack):
    push(indent_stack, level)
    emit INDENT
  пока level < top(indent_stack):
    pop(indent_stack)
    emit DEDENT
  если level == top(indent_stack):
    emit NEWLINE (если строка непустая)
в конце файла:
  пока len(indent_stack) > 1:
    pop(indent_stack)
    emit DEDENT
```

**Задачи:**
- [ ] Реализовать offside rule
- [ ] Обработка пустых строк (пропускаются)
- [ ] Обработка строк с только комментарием
- [ ] Обработка continuation (строка заканчивается оператором → продолжение)

**Тесты (не менее 20):**
- [ ] Простой блок с одним уровнем
- [ ] Вложенные блоки (2-3 уровня)
- [ ] Возврат на несколько уровней сразу
- [ ] Пустые строки внутри блока
- [ ] Комментарии внутри блока
- [ ] Файл без индентации
- [ ] Некорректная индентация (нечётное число пробелов) → ошибка

**Критерий готовности Фазы 1:**
Команда `echo "fac 0 = 1\nfac n = n * fac (n - 1)" | synoema-lex` выдаёт корректный поток токенов.

---

## ФАЗА 2: Парсер
**Срок: 3 недели**

### 2.1 AST определения (неделя 1)

```rust
enum Expr {
    Lit(Literal),
    Var(String),
    App(Box<Expr>, Box<Expr>),
    Lam(Vec<Pattern>, Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    UnaryMinus(Box<Expr>),
    Cond(Box<Expr>, Box<Expr>, Box<Expr>),  // ? -> :
    List(Vec<Expr>),
    ListComp(Box<Expr>, Vec<Generator>),
    Range(Box<Expr>, Box<Expr>),
    Block(Vec<Binding>, Box<Expr>),
    FieldAccess(Box<Expr>, String),
    Pipe(Box<Expr>, Box<Expr>),
    Constructor(String, Vec<Expr>),
}

enum Pattern {
    Wildcard,
    Var(String),
    Lit(Literal),
    Constructor(String, Vec<Pattern>),
    Cons(Box<Pattern>, Box<Pattern>),
}

enum Decl {
    FuncDef { name: String, equations: Vec<Equation> },
    TypeSig { name: String, ty: Type },
    TypeDef { name: String, params: Vec<String>, variants: Vec<Variant> },
    TraitDef { name: String, param: String, methods: Vec<TypeSig> },
    ImplDef { trait_name: String, ty: Type, methods: Vec<FuncDef> },
}
```

**Задачи:**
- [ ] Полное определение AST (все enum выше + вспомогательные)
- [ ] Pretty-printer для AST (отладочный вывод)
- [ ] Span-аннотации для сообщений об ошибках

### 2.2 Expression parser — Pratt parsing (неделя 2)

Pratt parsing (top-down operator precedence) — оптимальный алгоритм для expression-heavy языков.

**Задачи:**
- [ ] Pratt parser с 13 уровнями приоритета (из §3.6 спецификации)
- [ ] Prefix operators: unary minus, lambda (`\`), conditional (`?`)
- [ ] Infix operators: все бинарные операторы
- [ ] Postfix: field access (`.name`)
- [ ] Function application: juxtaposition (наивысший приоритет)
- [ ] Группировка: `( expr )`
- [ ] Списки: `[e1 e2 e3]`, `[e | x <- xs, guard]`, `[a..b]`
- [ ] Block expressions: INDENT bindings expr DEDENT

**Тесты (не менее 30):**
- [ ] Каждый бинарный оператор с правильным приоритетом
- [ ] `f x + 1` → `(f x) + 1` (application > addition)
- [ ] `a |> f |> g` → `g (f a)` (left-associative pipe)
- [ ] `? a -> b : ? c -> d : e` → правильная вложенность
- [ ] `\x -> \y -> x + y` → правильная ассоциативность
- [ ] Сложные выражения: `xs |> filter even |> map (\x -> x * 2) |> sum`

### 2.3 Declaration parser (неделя 3)

**Задачи:**
- [ ] Парсинг определений функций: `name patterns = expr`
- [ ] Множественные уравнения (pattern matching): группировка по имени
- [ ] Парсинг type signatures: `name : type`
- [ ] Парсинг ADT definitions: `Name params = Variant1 | Variant2`
- [ ] Парсинг trait/impl (заглушки для MVL — минимальная реализация)
- [ ] Парсинг top-level программы: последовательность деклараций
- [ ] Качественные сообщения об ошибках с указанием позиции

**Тесты (не менее 20):**
- [ ] Простая функция без паттернов
- [ ] Функция с множественными уравнениями (fib, fac)
- [ ] ADT: Maybe, List, Shape
- [ ] Type signature + определение
- [ ] Блок с локальными привязками
- [ ] Ошибочный ввод → понятное сообщение

**Критерий готовности Фазы 2:**
Все примеры из §7 спецификации (факториал, map, quicksort, fizzbuzz) парсятся в корректный AST.

---

## ФАЗА 3: Система типов
**Срок: 3 недели**

### 3.1 Инфраструктура типов (неделя 1)

**Задачи:**
- [ ] Представление типов:
  ```rust
  enum Type {
      Var(TypeVar),           // α, β, γ...
      Con(String),            // Int, Float, Bool...
      App(Box<Type>, Box<Type>), // Maybe Int, List a...
      Arrow(Box<Type>, Box<Type>), // a -> b
  }
  ```
- [ ] Типовые переменные: генератор свежих переменных
- [ ] Подстановки (Substitution): `HashMap<TypeVar, Type>`
- [ ] Композиция подстановок
- [ ] Свободные типовые переменные: `ftv(τ)`, `ftv(Γ)`
- [ ] Применение подстановки к типу, к окружению

### 3.2 Unification и Algorithm W (неделя 2)

**Задачи:**
- [ ] Occurs check: `α ∉ ftv(τ)` (предотвращение бесконечных типов)
- [ ] Robinson unification: `unify(τ₁, τ₂) → Result<Subst, TypeError>`
- [ ] Algorithm W: `infer(Γ, expr) → Result<(Subst, Type), TypeError>`
  - [ ] Литералы → фиксированные типы
  - [ ] Переменные → instantiate из окружения
  - [ ] Application → unification τ₁ с (τ₂ → α)
  - [ ] Lambda → свежая переменная для аргумента
  - [ ] Let-привязки → generalize
  - [ ] Conditional → обе ветки одного типа, guard : Bool
  - [ ] BinOp → специализированные правила для каждого оператора
  - [ ] List → все элементы одного типа
  - [ ] Pipe → desugaring в application

### 3.3 ADT и Pattern Matching (неделя 3)

**Задачи:**
- [ ] Типизация конструкторов: `Just : a → Maybe a`
- [ ] Типизация pattern matching:
  - Каждый паттерн порождает типовые ограничения
  - Все уравнения функции совместимы по типам
- [ ] Exhaustiveness check (предупреждение если паттерны неполные)
- [ ] Понятные сообщения об ошибках типизации:
  ```
  Error at line 5, col 12:
    Type mismatch in function 'add':
      Expected: Int
      Found:    String
    In expression: x ++ y
  ```

**Тесты (не менее 40):**
- [ ] Вывод типов для всех примеров из §7 спецификации
- [ ] Полиморфизм: `id x = x` → `∀a. a → a`
- [ ] Let-полиморфизм: `id` используется с разными типами
- [ ] ADT: Maybe, List — конструкторы типизируются корректно
- [ ] Ошибки: type mismatch, undefined variable, infinite type
- [ ] Pipe: `[1 2 3] |> map double |> sum` → `Int`

**Критерий готовности Фазы 3:**
Все примеры из §7 проходят type check. Ошибочные программы отвергаются с понятными сообщениями.

---

## ФАЗА 4: Интерпретатор и REPL
**Срок: 2 недели**

### 4.1 Tree-walking интерпретатор (неделя 1)

**Задачи:**
- [ ] Environment: `HashMap<String, Value>` с вложенными scope
- [ ] Eval по правилам из §5 спецификации:
  - Литералы, переменные, application, lambda
  - BinOp (все 15+ операторов)
  - Conditional (short-circuit)
  - Let-блоки
  - Pattern matching (match engine)
  - List operations
  - Рекурсия (окружение расширяется до вычисления)
- [ ] Встроенные функции: `print`, `show`, `length`, `head`, `tail`

### 4.2 REPL (неделя 2)

**Задачи:**
- [ ] Read-Eval-Print Loop с rustyline (readline библиотека)
- [ ] Ввод многострочных выражений (блоки с индентацией)
- [ ] Вывод типа каждого выражения после вычисления:
  ```
  synoema> fac 5
  120 : Int

  synoema> map (\x -> x * 2) [1 2 3]
  [2 4 6] : List Int

  synoema> :type map
  map : (a -> b) -> List a -> List b
  ```
- [ ] Команды: `:type expr`, `:quit`, `:load file`, `:help`
- [ ] Загрузка файлов: `synoema run file.sno`

**Тесты (не менее 30):**
- [ ] Все примеры из §7 выполняются корректно
- [ ] Рекурсия: fac 20 → правильный результат
- [ ] Pattern matching: all branches
- [ ] List comprehension: [x * x | x <- [1..10], x % 2 == 0]
- [ ] Ошибки runtime: деление на ноль, неполный pattern match

**Критерий готовности Фазы 4:**
REPL работает. Все 4 примера из §7 выполняются. Можно писать и запускать программы из файлов.

**🎯 MILESTONE 1: Working Language (Фазы 0-4)**
На этой точке у нас есть **работающий интерпретируемый язык**.
Можно публиковать, собирать фидбек, привлекать контрибьюторов.

---

## ФАЗА 5: BPE-верификация и токенные бенчмарки
**Срок: 1 неделя**

### 5.1 Автоматическая BPE-верификация

**Задачи:**
- [ ] Python-скрипт: tiktoken + sentencepiece для подсчёта токенов
- [ ] Автоматическая проверка: все операторы Synoema = 1 BPE-токен
- [ ] Регрессионный тест: если грамматика меняется — BPE-тест ломается

### 5.2 Токенные бенчмарки

**Задачи:**
- [ ] 20 задач из HumanEval: решения на Synoema + Python + Haskell
- [ ] Автоматический подсчёт токенов для каждого решения
- [ ] Генерация таблицы сравнения и графиков
- [ ] Публикация результатов в README

**Критерий:** средняя экономия ≥ 35% vs Python.

---

## ФАЗА 6: Core IR и десахаризация
**Срок: 2 недели**

### 6.1 Core IR (System F)

Промежуточное представление между AST и LLVM IR.

```rust
enum CoreExpr {
    Var(String),
    Lit(Literal),
    App(Box<CoreExpr>, Box<CoreExpr>),
    Lam(String, Type, Box<CoreExpr>),    // типизированная лямбда
    Let(String, Box<CoreExpr>, Box<CoreExpr>),
    Case(Box<CoreExpr>, Vec<Alt>),        // единый pattern match
    Con(String, Vec<CoreExpr>),
    PrimOp(PrimOp, Vec<CoreExpr>),        // примитивные операции
}
```

**Задачи:**
- [ ] Определение Core IR
- [ ] Десахаризация AST → Core:
  - Множественные уравнения → единый `Case`
  - `? -> :` → `Case` с Bool паттернами
  - `|>` → `App`
  - `>>` → вложенный `Lam`
  - List comprehension → `concatMap` / `filter`
  - Блоки → вложенные `Let`
- [ ] Сохранение типовой информации (из Фазы 3)
- [ ] Pretty-printer для Core IR (отладка)

### 6.2 Оптимизации на Core IR

**Задачи:**
- [ ] Инлайнинг мелких функций
- [ ] Constant folding (1 + 2 → 3)
- [ ] Dead code elimination
- [ ] Eta-редукция (`\x -> f x` → `f`)
- [ ] Beta-редукция (подстановка let-привязок)

**Критерий:** Core IR корректно представляет все программы, десахаризация проходит тесты.

---

## ФАЗА 7: LLVM Code Generation
**Срок: 4-6 недель**

### 7.1 Инфраструктура LLVM (неделя 1)

**Задачи:**
- [ ] Подключить `inkwell` (Rust bindings для LLVM)
- [ ] Module / Function / BasicBlock creation
- [ ] Генерация `main` entry point
- [ ] Hello World: Synoema → LLVM IR → executable
- [ ] Makefile / build script для компиляции `.ll` → binary

### 7.2 Кодогенерация выражений (недели 2-3)

**Задачи:**
- [ ] Литералы → LLVM constants
- [ ] Арифметические операции → LLVM `add`, `mul`, `sdiv` etc.
- [ ] Сравнения → LLVM `icmp`
- [ ] Условия → LLVM `br` (conditional branch)
- [ ] Функции → LLVM functions с правильными типами
- [ ] Вызов функций → LLVM `call`
- [ ] Рекурсия → обычный `call` (tail call optimization позже)

### 7.3 Замыкания и ADT (недели 3-4)

**Задачи:**
- [ ] Замыкания: представление как struct { function_ptr, env_ptr }
- [ ] Мономорфизация: для каждого конкретного типа — своя функция
- [ ] ADT: tagged union (tag byte + payload)
- [ ] Pattern matching → series of `icmp` + `br`
- [ ] Списки: linked list или array representation

### 7.4 Управление памятью (недели 4-5)

**Задачи:**
- [ ] Начальная версия: простой arena allocator (bump allocation)
- [ ] Строки: reference-counted или static
- [ ] Списки: heap-allocated cons cells с RC
- [ ] Region inference (базовая версия): определение lifetime для let-блоков

### 7.5 Стандартные операции (неделя 6)

**Задачи:**
- [ ] print / show → вызов C `printf`
- [ ] String operations → libc или собственная реализация
- [ ] List operations (map, filter, fold) → скомпилированные функции
- [ ] Entry point: `main` → вызов пользовательского `main`
- [ ] Return code

**Тесты:**
- [ ] Все примеры из §7 компилируются и дают правильные результаты
- [ ] Performance benchmark: fib 35 vs C/Python/Haskell
- [ ] Valgrind: нет memory leaks

**Критерий готовности Фазы 7:**
`synoema compile fib.sno -o fib && ./fib` выдаёт правильный результат.

**🎯 MILESTONE 2: Working Compiler (Фазы 5-7)**
На этой точке Synoema компилируется в native код через LLVM.

---

## ФАЗА 8: Constrained Decoding интеграция
**Срок: 3 недели**

### 8.1 Экспорт грамматики (неделя 1)

**Задачи:**
- [ ] Конвертер: Synoema EBNF → GGML BNF (формат XGrammar)
- [ ] Конвертер: Synoema EBNF → Outlines JSON Schema
- [ ] Тест: грамматика принимает все валидные программы из тест-сьюта
- [ ] Тест: грамматика отвергает все невалидные строки

### 8.2 Интеграция с SGLang (неделя 2)

**Задачи:**
- [ ] SGLang plugin: Synoema как выходной формат
- [ ] Тест: LLM генерирует Synoema-код с constrained decoding
- [ ] Замер: compilation rate (% программ, проходящих парсер)
- [ ] Замер: сравнение с unconstrained генерацией

### 8.3 Type-constrained decoding (неделя 3)

**Задачи:**
- [ ] Экспорт type environment как дополнительных ограничений
- [ ] Proof of concept: после `x : Int`, только числовые операции допустимы
- [ ] Замер: сравнение functional correctness с/без type constraints

**🎯 MILESTONE 3: LLM-Native Compiler (Фаза 8)**
На этой точке LLM может генерировать гарантированно-корректный Synoema-код.

---

## ФАЗА 9: Расширения языка
**Срок: 4-6 недель (итеративно)**

### 9.1 Записи (records) + row polymorphism
- [ ] Синтаксис: `.field value`
- [ ] Row types: `{name : String, age : Int | r}`
- [ ] Доступ к полям: `r.name`
- [ ] Обновление: `{r | name = "Bob"}`

### 9.2 Модули
- [ ] `mod Name` — объявление
- [ ] `use Name (a b c)` — импорт
- [ ] Раздельная компиляция (по модулям)

### 9.3 Effects / IO
- [ ] `@io` маркер для IO-операций
- [ ] `<-` для bind в эффектных блоках
- [ ] File I/O, HTTP (через FFI)

### 9.4 FFI
- [ ] `@native "c_function"` — вызов C-функций
- [ ] Автоматическая маршалинг типов Synoema ↔ C

### 9.5 Type classes (полная реализация)
- [ ] `trait` с дефолтными методами
- [ ] `impl` с constraints
- [ ] Стандартные: Eq, Ord, Show, Num

---

## ФАЗА 10: Оптимизация производительности
**Срок: 4 недели**

### 10.1 Whole-program optimization
- [ ] Дефункционализация замыканий
- [ ] Unboxing примитивных типов
- [ ] Strictness analysis (обнаружение ненужной ленивости)
- [ ] Специализация полиморфных функций

### 10.2 Продвинутое управление памятью
- [ ] Region inference (Tofte-Talpin алгоритм)
- [ ] Escape analysis: что аллоцировать на стеке vs heap
- [ ] Reference counting для escape-случаев

### 10.3 LLVM-оптимизации
- [ ] Tail call optimization
- [ ] SIMD vectorization (для числовых операций на списках)
- [ ] Link-time optimization (LTO)

### 10.4 Бенчмарки
- [ ] Computer Language Benchmarks Game: 5 задач
- [ ] Сравнение: Synoema vs C vs Rust vs Haskell vs OCaml
- [ ] Цель: ≤ 2x от C на числовых задачах, ≤ 3x на общих

**🎯 MILESTONE 4: Production Compiler (Фазы 9-10)**

---

## Сводная таблица

| Фаза | Что | Срок | Результат |
|------|-----|------|-----------|
| 0 | Инфраструктура | 1 нед | Репозиторий, CI, структура |
| 1 | Лексер | 2 нед | Поток токенов из исходника |
| 2 | Парсер | 3 нед | AST из потока токенов |
| 3 | Типы | 3 нед | Type inference для всего MVL |
| 4 | Интерпретатор | 2 нед | REPL, запуск программ |
| **M1** | **Working Language** | **~11 нед** | **Интерпретируемый Synoema** |
| 5 | BPE-верификация | 1 нед | Подтверждение токенной экономии |
| 6 | Core IR | 2 нед | Промежуточное представление |
| 7 | LLVM codegen | 5 нед | Компиляция в native |
| **M2** | **Working Compiler** | **~19 нед** | **Нативный Synoema** |
| 8 | Constrained decoding | 3 нед | LLM генерирует Synoema |
| **M3** | **LLM-Native** | **~22 нед** | **Полная LLM-интеграция** |
| 9 | Расширения | 5 нед | Records, modules, IO, FFI |
| 10 | Оптимизации | 4 нед | C++/Rust-уровень |
| **M4** | **Production** | **~31 нед** | **Готов к использованию** |

---

## Приоритеты и риски

### Наибольшие риски:

1. **Замыкания + LLVM** (Фаза 7.3) — представление closures в LLVM нетривиально. Митигация: изучить реализацию в MLton, GHC LLVM backend, Rust.

2. **Region inference** (Фаза 10.2) — сложный алгоритм, может потребовать упрощения. Митигация: начать с простого RC, region inference как оптимизация.

3. **Cold start для LLM** (Фаза 8) — LLM не обучена на Synoema-коде. Митигация: few-shot промпты с примерами из §7, fine-tuning на синтетическом корпусе.

### Что можно параллелить:

- Фаза 5 (BPE-верификация) — параллельно с Фазами 2-3
- Документация и сайт — параллельно с любой фазой
- Привлечение контрибьюторов — после Milestone 1

### Когда публиковать:

- **После Фазы 0:** пост на HN/Reddit с описанием концепции и ссылкой на GitHub
- **После Milestone 1:** демо REPL, приглашение контрибьюторов
- **После Milestone 2:** release v0.1, бенчмарки производительности
- **После Milestone 3:** научная статья для Workshop on LLM Code Generation

---

*Synoema Compiler Roadmap v1.0*
*Март 2026*
