# Synoema — Полное состояние проекта

> Этот документ содержит всё необходимое для продолжения разработки проекта Synoema
> любым LLM-разработчиком. Читай этот файл ПЕРВЫМ.

---

## 0. Быстрый статус (апрель 2026)

**Версия: 0.1.0-alpha.2** — alpha-стадия, синтаксис и API могут меняться. Политика версий: [docs/versioning.md](../docs/versioning.md)

- **937 тестов**, все зелёные, 0 warnings
- **Prelude:** `lang/prelude/prelude.sno` — Result type + combinators (map_ok, map_err, unwrap, unwrap_or, is_ok, is_err, and_then)
- **`error : String -> a`** builtin — runtime panic в interpreter + JIT
- **npm дистрибуция:** `npx synoema-mcp` — 5 пакетов (launcher + 4 платформенных бинарника), версии синхронизированы из git tag через CI
- **Phases 9.2–23** завершены + TCO в JIT + String stdlib в JIT + Doc-as-Code + LLM Cost Reduction v1 + Region Inference + Doc Extraction API
- JIT поддерживает: int, bool, float, string, list, closures, records, record patterns, **record update**, modules, ADTs, ==, !=, list comprehensions, IO (print/readline), show (все типы), type class dispatch, higher-order stdlib, **self-recursive TCO**, **string stdlib** (str_slice/find/starts_with/trim/len/json_escape)
- Interpreter: всё вышеперечисленное + сетевые примитивы (tcp_listen/accept, fd_*/popen)
- Type checker: Hindley-Milner + row polymorphism + linear types (LinearArrow) + **type aliases** (`type Pos = {x: Int, y: Int}`)
- Система диагностики: synoema-diagnostic, структурированные ошибки с span, JSON/human рендереры, **LLM error feedback** (llm_hint, fixability, did_you_mean для top-12 ошибок)
- **Doc Extraction API:** `synoema doc --format json` — structured JSON output с doc-comments и inline-комментариями; MCP tool `doc_query` для LLM-запросов документации из .sno файлов
- **Error recovery:** `parse_recovering()` и `typecheck_recovering()` — сбор всех ошибок за один проход
- **Feedback loop:** `tools/llm/feedback_loop.py` — generate → check → enrich → retry pipeline
- **Stdlib catalog:** `docs/llm/stdlib.md` — машиночитаемый каталог всех builtins с типами
- **Small Model Quality Stack Phase 1:**
  - `docs/llm/synoema-compact.md` — compact reference ~900 токенов (vs 1800 full) для малых моделей 4B–32B
  - `docs/llm/templates/` — 5 task-specific prompt templates (arithmetic, lists, adt-patterns, records-maps, string-io) по 540–730 токенов каждый
  - `docs/llm/templates/gotcha-map.json` — feature → gotcha ID mapping для динамической инъекции предупреждений
  - Phase D benchmark (`benchmarks/runner/src/phases/size.rs`) — multi-model × multi-config × multi-pass тестирование малых моделей через ollama

---

## 1. Что такое Synoema

Synoema [sy-NO-e-ma] — язык программирования, оптимизированный для генерации кода
языковыми моделями (LLM). Название: σύν (вместе) + νόημα (содержание мысли).

**Три ключевых преимущества:**
- **-15% токенов** vs Python в среднем, до 52% на алгоритмических задачах (16 автоматизированных бенчмарков, tiktoken cl100k_base)
- **3× медиана скорости** vs Python (Cranelift JIT, 12 бенчмарков, диапазон 2.1×–28×)
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
│   ├── examples/               # 44 программы .sno (алгоритмы, структуры, паттерны, утилиты)
│   ├── benchmarks/
│   │   ├── token-count/        # 12 программ, BPE-бенчмарк
│   │   └── performance/        # JIT vs Python бенчмарки
│   ├── tools/
│   │   ├── bpe-verify/         # Верификация 33/33 операторов = 1 BPE-токен
│   │   └── constrained/        # synoema.gbnf + SGLang интеграция
│   ├── tests/                  # Интеграционные тесты (lexer, parser)
│   └── spec/                   # Формальная спецификация
│
├── benchmarks/                    # ← СРАВНИТЕЛЬНЫЙ BENCHMARK SUITE
│   ├── runner/                 # Rust CLI (synoema-bench): orchestration, telemetry, reports
│   ├── scripts/                # Python: token_count.py (tiktoken), llm_generate.py (OpenRouter)
│   ├── tasks/                  # 16 задач × 5 языков (sno, py, js, ts, cpp)
│   └── results/                # Результаты прогонов (gitignored)
│
├── docs/
│   ├── benchmarks.md           # Документация benchmark suite
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
| Тестов | 937 (все зелёные) |
| Warnings | 0 |
| Крейтов | 8 (добавлен synoema-diagnostic) |
| Примеров | 14 программ (.sno) |
| BPE-aligned операторов | 33/33 |
| Экономия токенов vs Python | 15% среднее (до 52%, 16 задач) |
| Ускорение vs Python (JIT) | 3× медиана (2.1×–28×, 12 задач) |
| GBNF-грамматика | 162 строки, 48 правил |

## 4. Что работает

### Interpreter (`synoema run`) — ВСЕ фичи:
- Pattern matching (single-arg, multi-arg, nested, cons, singleton list `[x]`)
- Closures, higher-order functions (map, filter, fold, zip, index, take, drop, reverse)
- Списки: `[1 2 3]`, cons `x:xs`, concat `++`, comprehensions `[x | x <- xs, p x]`
- Строки: `"hello"`, конкатенация `++`, `show`, интерполяция `"${expr}"`
- Условия: `? cond -> then : else`
- Where-блоки: `f x = y + z` / `  y = ...` / `  z = ...`
- Let-polymorphism: `id` как `Int → Int` и `Bool → Bool` в одной программе
- Pipe: `x |> f |> g`
- Рекурсия: factorial, fibonacci, quicksort, ackermann
- **Records (Phase 9.4):** `{x = 3, y = 4}`, field access `p.x`, pattern matching
- **Record update:** `{...r, x = 10}` — копировать record, перезаписать поля (interpreter + JIT)
- **Modules (Phase 9.5):** `mod Math`, `use Math (square pi)` — lexical namespacing
- **Multi-file imports:** `import "path.sno"` — recursive loading, cycle detection, diamond caching
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
- **Self-recursive TCO:** tail calls to self compiled as jumps (loop header pattern), O(1) stack for tail-recursive functions
- **String stdlib:** `str_slice`, `str_find`, `str_starts_with`, `str_trim`, `str_len`, `json_escape` — all via FFI, 13 tests
- Heap-allocated linked list + string + record + ConNode + FloatNode runtime
- **Arena hardening (Memory Management v2):** overflow tracking + cleanup, arena_save/restore for per-scope reset, overflow warning
- **Streaming file I/O:** `fd_open` / `fd_open_write` for line-by-line file processing (interpreter)

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

### Performance (JIT vs CPython 3.12, 12 задач, median of 5 runs):
```
fibonacci:      28.2× faster
factorial:       4.2× faster
gcd:             3.5× faster
collatz:         3.1× faster
quicksort:       2.7× faster
matrix_mult:     2.1× faster
────────────────────────────
Median (12):     3.0× faster
```

## 6. Известные баги

0 известных багов, 937/937 тестов зелёные.

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
cargo run -p synoema-repl -- init myapp                    # Scaffold проекта
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

## 11. Small Model Quality Stack — Roadmap

Стратегия повышения качества генерации Synoema на малых моделях (4B–32B).
Исследование проведено в апреле 2026 на основе 15+ научных работ (PLDI 2025, NeurIPS 2024, ICLR 2025, ACL 2025).

### Phase 1: Quick Wins ✅ (завершено)

- `docs/llm/synoema-compact.md` — compact reference ~900 токенов (gotcha-first ordering)
- `docs/llm/templates/` — 5 task-specific prompt templates (540–730 tok each)
- `docs/llm/templates/gotcha-map.json` — feature → gotcha ID mapping
- `benchmarks/runner/src/phases/size.rs` — Phase D benchmark (multi-model × multi-config × multi-pass)
- CLI: `--phases size`, `--size-models`

### Phase 2: QLoRA Fine-Tune (ожидание: +40% run rate, 2–3 недели)

**Требует подтверждения перед началом.**

- Синтетический корпус: 10K–50K verified (instruction, code) pairs
  - Генерация frontier-моделью (Claude/GPT-4o) с reference контекстом
  - Верификация: `synoema run` → parse ✓ typecheck ✓ run ✓
  - Dedup по AST similarity, баланс по сложности
- QLoRA fine-tune Qwen2.5-Coder 7B (Unsloth, ~5 GB VRAM)
  - r=16, alpha=32, 3 epochs, lr=2e-4, cosine scheduler
  - Hardware: 1× RTX 4090 или A100
- A/B benchmark: fine-tuned vs in-context на Phase D
- Integrate DOMINO (arxiv:2403.06988) для zero-overhead grammar constraints

### Phase 3: Type-Constrained Decoding (ожидание: -50% type errors, 4–6 недель)

**Требует подтверждения. Research-heavy.**

Основано на: Mündler et al., PLDI 2025 (arxiv:2504.09246) — type-constrained decoding для TypeScript.
Synoema (Hindley-Milner, чистый, BPE-aligned) значительно проще для type-constrain чем TypeScript.

- Incremental type checker: extend `synoema-types` для partial prefix → type constraints
- XGrammar logit processor: type mask computation на каждом шаге генерации
- Think-then-Constrain integration (arxiv:2601.07525, Jan 2026): свободное "думание" до trigger-токена, потом grammar+types kick in
- IterGen backtracking (ICLR 2025): forward/backward generation с KV-cache reuse

### Phase 4: Self-Play RL (ожидание: push к 80%+ run rate, 4+ недель)

**Требует подтверждения. Наиболее research-тяжёлая фаза.**

Основано на: Sol-Ver (arxiv:2502.14948, March 2026), CoCoS (arxiv:2505.23060, 2025).
Ключевое преимущество Synoema: compiler = perfect verifier (ground truth без LLM-as-judge).

- Sol-Ver adaptation: LLM-as-solver + compiler-as-verifier + LLM-as-test-generator
- CoCoS-style RL для self-correction на 1B–7B моделях
  - Accumulated reward с discount factor
  - Fine-grained per-turn reward
- Ожидание: +19–35% code generation quality (на основе results Sol-Ver и CoCoS)

### Ожидаемый кумулятивный эффект (7B модель)

| Стек | Syntax rate | Type rate | Run rate | Latency |
|------|------------|-----------|----------|---------|
| Baseline (in-context, no GBNF) | ~60% | ~35% | ~20% | ×1.0 |
| Phase 1 (compact + templates + GBNF + multipass) | 100% | ~65% | ~45% | ×2.5 |
| Phase 2 (+ QLoRA fine-tune + DOMINO) | 100% | ~80% | ~65% | ×1.5 |
| Phase 3 (+ type-constrain + IterGen) | 100% | ~95% | ~75% | ×1.2 |
| Phase 4 (+ Sol-Ver self-play) | 100% | ~95% | ~80%+ | ×1.0 |

### Ключевые research-ссылки

- DOMINO (ICML 2024): arxiv:2403.06988 — zero-overhead BPE-aligned grammar constraints
- Pre³ (ACL 2025): arxiv:2506.03887 — deterministic PDA, +36% throughput vs XGrammar
- Grammar-Aligned Decoding (NeurIPS 2024): arxiv:2405.21047 — unbiased grammar-constrained sampling
- Type-Constrained Code Gen (PLDI 2025): arxiv:2504.09246 — incremental type checker for code LLMs
- Think-then-Constrain (Jan 2026): arxiv:2601.07525 — hybrid free/constrained decoding, +27% accuracy
- IterGen (ICLR 2025): arxiv:2410.07295 — grammar backtracking with KV-cache reuse, +18.5%
- CoCoS (2025): arxiv:2505.23060 — self-correcting code gen for 1B models, +35.8% MBPP
- Sol-Ver (March 2026): arxiv:2502.14948 — self-play solver-verifier, +19.6% code gen on 8B
- Qwen2.5-Coder: qwenlm.github.io — 7B/14B/32B, 92 languages, 128K context
- Qwen3-Coder-Next (2026): 3B active (MoE), SWE-Bench >70%
- Unsloth: github.com/unslothai/unsloth — 2× faster QLoRA, 70% less VRAM
