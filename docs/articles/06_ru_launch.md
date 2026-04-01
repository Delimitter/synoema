# Synoema: первый язык программирования, спроектированный для LLM

## 264 теста, 7055 строк Rust, 46% экономия токенов, 4.4× быстрее Python

---

> **TL;DR.** Synoema — язык программирования, оптимизированный для генерации кода языковыми моделями. На 46% меньше токенов, чем Python. 4.4× быстрее Python (Cranelift JIT). 100% синтаксическая корректность через GBNF-грамматику. Hindley-Milner вывод типов без аннотаций. Open source, MIT License.

---

## Зачем ещё один язык

Каждый день миллионы разработчиков просят LLM написать код на Python. Модель генерирует `def`, `return`, `if/elif/else`, запятые в списках — десятки токенов синтаксического оверхеда, которые не несут смысловой нагрузки, но стоят денег и вычислений.

За последние месяцы мы провели исследование и обнаружили три фундаментальных проблемы:

1. **Python тратит 46% токенов впустую** — на синтаксис, который не несёт семантики.
2. **33.6% ошибок LLM-кода — типовые** — и их можно устранить formально.
3. **Интерпретация медленная** — сгенерированный код можно компилировать в нативный за миллисекунды.

Synoema [sy-NO-e-ma] решает все три проблемы. Название от греческих σύν (вместе) + νόημα (содержание мысли) — язык *совместного понимания* между человеком и LLM.

## Быстрый старт

```bash
# Установка
git clone https://github.com/synoema/synoema
cd synoema && cargo build --release

# Запуск примера (интерпретатор — все фичи)
synoema run examples/quicksort.sno
# → [1 2 3 4 5 6 7 8 9]

# JIT-компиляция (нативная скорость)
synoema jit examples/factorial.sno
# → 3628800

# Вычислить выражение
synoema eval "6 * 7"
# → 42

# Интерактивный REPL
synoema
synoema> fac 0 = 1
synoema> fac n = n * fac (n - 1)
synoema> fac 10
3628800
```

## Как выглядит код

```
-- Факториал
fac 0 = 1
fac n = n * fac (n - 1)

-- QuickSort
qsort [] = []
qsort (p:xs) = qsort lo ++ [p] ++ qsort hi
  lo = [x | x <- xs , x <= p]
  hi = [x | x <- xs , x > p]

-- Map
map f [] = []
map f (x:xs) = f x : map f xs

-- FizzBuzz
fizz n = ? n % 15 == 0 -> "FizzBuzz"
       : ? n % 3 == 0  -> "Fizz"
       : ? n % 5 == 0  -> "Buzz"
       : show n
```

Синтаксис: pattern matching, `? cond -> then : else`, списки без запятых `[1 2 3]`, list comprehensions `[x | x <- xs , p x]`, `where`-блоки через отступы.

## Ключевые числа

### Токенная эффективность: -46% vs Python

| Программа | Synoema | Python | Экономия |
|-----------|---------|--------|----------|
| Factorial | 16 | 29 | 45% |
| Map | 20 | 42 | 52% |
| QuickSort | 51 | 83 | 39% |
| FizzBuzz | 44 | 64 | 31% |
| Filter | 27 | 67 | 60% |
| **Итого (12 программ)** | **332** | **615** | **46%** |

Из-за квадратичной стоимости attention: 46% меньше токенов ≈ **71% меньше вычислений**.

### Производительность: 4.4× быстрее Python

| Бенчмарк | Python | Synoema JIT | Ускорение |
|----------|--------|-------------|-----------|
| fib(30) | 277 мс | 47 мс | 5.9× |
| gcd (100K) | 143 мс | 83 мс | 1.7× |
| collatz (10K) | 505 мс | 90 мс | 5.6× |
| **Среднее** | | | **4.4×** |

### BPE-alignment: 33/33 операторов = 1 BPE-токен

Каждый оператор Synoema кодируется ровно в 1 BPE-токен на cl100k_base (GPT-4/Claude) и o200k_base (GPT-4o). Нет мостовых токенов. Нет синтаксического оверхеда.

## Constrained Decoding

GBNF-грамматика Synoema (41 правило) подключается к любому inference-движку:

```python
# SGLang / vLLM / XGrammar
response = client.chat.completions.create(
    model="default",
    messages=[{"role": "user", "content": "Write quicksort in Synoema"}],
    extra_body={"ebnf": open("synoema.gbnf").read()},
)
# Результат: 100% синтаксически валидный код
```

```bash
# llama.cpp
./main -m model.gguf --grammar-file synoema.gbnf \
  -p "-- Fibonacci in Synoema:" -n 128
```

## Архитектура

7 crates, 7055 строк Rust, 264 теста:

| Компонент | Строк | Тестов | Что делает |
|-----------|-------|--------|-----------|
| synoema-lexer | 706 | 80 | Токенизация, offside rule |
| synoema-parser | 1398 | 36 | Pratt parser, 15 типов выражений |
| synoema-types | 1453 | 42 | Hindley-Milner, let-polymorphism |
| synoema-core | 969 | 26 | Core IR (System F), десахаризация |
| synoema-eval | 1314 | 46 | Интерпретатор: closures, списки, ADT |
| synoema-codegen | 944 | 34 | Cranelift JIT → нативный x86-64 |
| synoema-repl | 271 | — | CLI: run / jit / eval / REPL |

## Roadmap

- [x] Lexer, Parser, Type System, Interpreter, REPL
- [x] Cranelift JIT (integers + lists)
- [x] BPE Benchmarks (46% vs Python)
- [x] GBNF Grammar + SGLang integration
- [ ] Closures в JIT (map, filter через нативный код)
- [ ] Records + row polymorphism
- [ ] Модули (`mod`/`use`)
- [ ] IO / Effects
- [ ] LLVM backend (`--backend llvm`)
- [ ] VS Code расширение
- [ ] Web Playground

## Попробовать

```bash
git clone https://github.com/synoema/synoema
cd synoema
cargo test           # 264 теста
cargo run -p synoema-repl -- run examples/quicksort.sno
cargo run -p synoema-repl -- jit examples/factorial.sno
```

Лицензия: MIT. Контрибьюции приветствуются.

---

*Шестая статья из серии «Токенная экономика кода». Предыдущие статьи: [#1 Стоимость токенов], [#2 Анатомия BPE], [#3 Constrained Decoding], [#4 Компиляция для LLM], [#5 Hindley-Milner].*
