---
id: design
type: design
status: done
---

# Design: Constrained Decoding E2E Pipeline

## Approach: Docker-Based Validation Harness

### Architecture

```
┌─────────────────────────────────────────────────────┐
│                 docker-compose.yml                   │
├───────────────────┬─────────────────────────────────┤
│                   │                                  │
│   ┌───────────┐   │   ┌──────────────────────────┐  │
│   │  SGLang   │   │   │   synoema-validator      │  │
│   │  server   │◄──┼──▶│   (Python + cargo)       │  │
│   │           │   │   │                          │  │
│   │ + GBNF    │   │   │  1. Генерация программ   │  │
│   │   grammar │   │   │  2. Parse + Typecheck    │  │
│   └───────────┘   │   │  3. Метрики              │  │
│                   │   └──────────────────────────┘  │
└───────────────────┴─────────────────────────────────┘
```

### Option A (chosen): SGLang + OpenAI-compatible API

SGLang поддерживает EBNF/GBNF через XGrammar (their default engine).
API pattern:
```python
response = client.chat.completions.create(
    model="default",
    messages=[{"role": "user", "content": prompt}],
    extra_body={"ebnf": grammar_text},
    max_tokens=256,
)
```

**Pros:**
- XGrammar claimed "near-zero overhead" for DCFG
- OpenAI-compatible API (стандартный интерфейс)
- Активно развивается

**Cons:**
- Требует GPU для реалистичных тестов
- Тяжёлый Docker image

### Option B (alternative): llama.cpp

```bash
./main -m model.gguf --grammar-file synoema.gbnf -p "Write factorial:"
```

**Pros:**
- CPU-friendly, легче запустить
- GBNF — нативный формат

**Cons:**
- Менее production-relevant
- Ограниченная модельная поддержка

### Decision: поддержать ОБА, начиная с llama.cpp для CI (CPU), SGLang для benchmarks (GPU)

## Validation Pipeline

```
    Prompt Database (10 tasks × 10 variations = 100 prompts)
         │
         ▼
    ┌────────────────┐
    │ Generate under  │─── timing ──▶ latency_ms
    │ GBNF constraint │
    └────────┬───────┘
             │ generated .sno
             ▼
    ┌────────────────┐
    │ Parse           │─── result ──▶ syntax_ok: bool
    │ (cargo run)     │
    └────────┬───────┘
             │ AST
             ▼
    ┌────────────────┐
    │ Type Check      │─── result ──▶ types_ok: bool
    │                 │─── errors ──▶ Vec<Diagnostic>
    └────────┬───────┘
             │
             ▼
    ┌────────────────┐
    │ Report          │─── JSON ───▶ results.json
    │ Generation      │─── Human ──▶ report.md
    └────────────────┘
```

## Prompt Database

10 задач по возрастающей сложности:

| # | Task | Complexity | Tests |
|---|------|-----------|-------|
| 1 | Constant/literal | Trivial | `main = 42` |
| 2 | Arithmetic | Simple | `double x = x * 2` |
| 3 | Factorial (recursion) | Medium | Pattern matching + recursion |
| 4 | List operations | Medium | map/filter/fold |
| 5 | String processing | Medium | Concatenation, show |
| 6 | Records | Medium-High | Field access, patterns |
| 7 | ADTs | High | Maybe/Either + matching |
| 8 | Higher-order | High | Closures, pipes |
| 9 | Quicksort | High | Full algorithm |
| 10 | Module + types | Very High | mod + type + impl |

Каждая задача × 10 формулировок = 100 промптов.

## Metrics Collected

```json
{
  "total_generated": 100,
  "syntax_correct": 100,
  "syntax_rate": 1.0,
  "type_correct": 73,
  "type_rate": 0.73,
  "avg_latency_ms": 245,
  "avg_tokens_generated": 42,
  "grammar_overhead_pct": 3.2,
  "errors_by_code": {
    "type_mismatch": 12,
    "unbound_variable": 8,
    "arity_mismatch": 7
  }
}
```

## CI Integration

GitHub Actions workflow:
1. On push to `tools/constrained/synoema.gbnf` or `lang/crates/synoema-parser/`
2. Build Synoema (cargo build)
3. Run llama.cpp with small model (TinyLlama) + GBNF
4. Generate 20 programs
5. Verify syntax correctness = 100%
6. Post results as PR comment
