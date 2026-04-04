# Spec: Article Series Structure

## Серия: "Token Economics of Code" (11 статей)

### Фаза 1: Теория (опубликовано)
| # | Заголовок (EN) | Статус | Платформа |
|---|---------------|--------|-----------|
| 1 | Why Every Token Costs More Than You Think | ✅ Published | Medium |
| 2 | The Anatomy of BPE: Why Python Wastes 46% of Tokens | ✅ Published | Medium |
| 3 | Type-Guided Constrained Decoding | ✅ Published | Medium |

### Фаза 2: Решение (обновить)
| # | Заголовок (EN) | Файл | Действие |
|---|---------------|------|----------|
| 4 | Compilation for LLMs: Cranelift JIT | 04_en_compilation.md | Update stats |
| 5 | Hindley-Milner for LLMs: Type Inference | 05_en_hindley_milner.md | Add practical examples |

### Фаза 3: Данные (новые)
| # | Заголовок (EN) | Файл | Тип |
|---|---------------|------|-----|
| 8 | Token Efficiency: 16 Algorithms, 5 Languages | 08_en_token_benchmark.md | NEW |
| 9 | JIT vs Interpreters: Runtime Benchmarks | 09_en_runtime_benchmark.md | NEW |
| 10 | Can LLMs Write Synoema? 10 Models, 9 Tasks | 10_en_llm_generation.md | NEW |
| 11 | The Real Cost: Token Savings for Teams | 11_en_cost_calculator.md | NEW |

### Фаза 4: Запуск (обновить)
| # | Заголовок (EN) | Файл | Действие |
|---|---------------|------|----------|
| 6 | Synoema: First Language Designed for LLMs | 06_en_launch.md | Major update |
| 7 | The Future of Code Generation | 07_en_future.md | Light update |

## Порядок публикации (после доработки)

| Неделя | Статья | Платформа | Hook |
|--------|--------|-----------|------|
| 1 | #8 Token Benchmark | Medium + dev.to | Hard data, tables |
| 2 | #4 JIT Compilation | Medium + dev.to | 4.4× faster |
| 3 | #9 Runtime Benchmark | Medium + dev.to | Charts, comparisons |
| 4 | #5 Hindley-Milner | Medium + dev.to | Zero-cost types |
| 5 | #10 LLM Generation | Medium + dev.to | Viral potential |
| 6 | #11 Cost Calculator | Medium + dev.to | Practical takeaway |
| 7 | #6 Launch | HN + Medium + dev.to | Show HN |
| 8 | #7 Future | Medium + dev.to | Vision piece |

## Формат: Medium vs dev.to

### Medium
- Заголовок: провокационный вопрос
- Intro: storytelling, personal hook
- Сноски (footnotes) для глоссария
- Графические embeds для данных
- CTA: "Follow for Part N"
- Серия: указывать "Part N of Token Economics of Code"

### dev.to
- Заголовок: конкретное обещание с числами
- Intro: TL;DR сразу
- Inline blockquotes для терминов (без footnotes)
- ASCII таблицы для данных
- Tags: #benchmark #rust #llm #programming
- CTA: "Try it: `cargo run ...`"
- Использовать dev.to Series feature

## Актуальные числа проекта (для обновлений)

| Метрика | Старое значение | Новое значение |
|---------|----------------|---------------|
| Tests | 264 | 890+ |
| LOC (Rust) | 7,055 | ~12,000 |
| Crates | 7 | 8 (+ synoema-diagnostic) |
| Phases completed | 9.2-15 | 9.2-21 |
| JIT features | int + lists | int, bool, float, string, list, closures, records, ADTs, modules, TCO |
| Prelude | — | Result type + combinators |
| MCP | — | synoema-mcp server, npm distribution |
| Diagnostics | — | Structured errors, JSON/human, LLM hints |
| Region inference | — | Memory management without GC |
| Benchmark suite | 12 tasks, 3 langs | 16 tasks, 5 langs, 3 phases |
