---
id: proposal
type: proposal
status: done
---

# Proposal: Constrained Decoding E2E Pipeline

## Problem Statement

Synoema имеет production-ready GBNF грамматику (48 правил, BPE-aligned) и BPE-verification tooling, но **нет работающего end-to-end pipeline** для constrained decoding с реальными LLM inference engines.

Текущее состояние:
- `synoema.gbnf` — формально специфицирована, детерминистическая CFG ✅
- `verify_bpe.py` — верификация токенов cl100k_base + o200k_base ✅
- `integration.py` — примеры curl/Python, но **не тестирует GBNF** (запускает cargo test вместо grammar validation)
- Нет E2E бенчмарков: throughput, latency, perplexity impact
- Нет CI/CD для grammar regression testing

Без реального E2E pipeline невозможно утверждать "100% syntactic correctness" — это теоретическое свойство, не проверенное на практике.

## Scope

### Phase 1: Validation Harness (~200 LOC Python)
- Docker-compose: SGLang/llama.cpp + Synoema grammar
- Генерация N программ under constrained decoding
- Автоматическая верификация: parse + typecheck каждой
- Метрики: syntax correctness rate, type correctness rate, generation speed

### Phase 2: Benchmark Suite (~150 LOC)
- 10 задач разной сложности (factorial → quicksort → ADT → records)
- Сравнение: constrained vs unconstrained generation
- Latency overhead от грамматики
- Token efficiency в реальной генерации

### Phase 3: CI Integration (~50 LOC)
- GitHub Actions workflow: при изменении GBNF → прогон E2E
- Regression detection: alerting при деградации correctness

## Prior Art

- Beurer-Kellner et al. (ICML 2024) "Domino" — bridge token misalignment (решено BPE alignment)
- Dong et al. (2024) — XGrammar: near-zero overhead for DCFG
- Mündler et al. (PLDI 2025) — type-constrained decoding: 74.8% error reduction

## Success Criteria

- [ ] Docker-compose поднимает SGLang + Synoema grammar за `docker compose up`
- [ ] Генерация 100 программ: ≥99% syntax correctness (цель: 100%)
- [ ] Генерация 100 программ: ≥70% type correctness (baseline measurement)
- [ ] Benchmark: latency overhead ≤10% vs unconstrained
- [ ] CI workflow: GBNF regression detection на каждый PR
- [ ] Документация: `docs/constrained-decoding-e2e.md`
- [ ] Метрики доступны в human-readable формате

## Non-Goals

- Type-directed generation (отдельная задача, зависит от этой)
- Production deployment (Kubernetes, monitoring) — следующий этап
- Поддержка всех inference engines (начинаем с SGLang или llama.cpp)
