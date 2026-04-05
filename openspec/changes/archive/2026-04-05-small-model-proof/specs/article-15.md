# Article 15: Workshop Paper

## Purpose

Write Article 15 in the educational series: "Does Language Design Reduce LLM Size Requirements?" — formatted as a workshop paper suitable for LLM4Code, COLM, or NeurIPS workshops.

## Requirements

### R1: Structure

Standard workshop paper format (4-6 pages):
1. Abstract (~150 words)
2. Introduction — thesis, motivation, contribution
3. Background — Synoema design, constrained decoding, token efficiency
4. Experimental Setup — models, languages, modes, tasks, metrics
5. Results — tables, charts, key findings
6. Analysis — why Synoema wins, decomposition of gains (GBNF vs tokens vs syntax)
7. Related Work — code generation benchmarks, constrained decoding, DSLs for LLMs
8. Conclusion

### R2: Key Claim

"A language designed for LLM code generation reduces model size requirements by N× compared to Python, as measured by minimum model size achieving 70% correctness on 30 code generation tasks."

### R3: File

`docs/articles/15_en_small_model_proof.md`

Russian version deferred (not in scope).

### R4: Data-Driven

All claims backed by Phase D benchmark data. Include actual numbers, not hypothetical.
Placeholder format `[RESULT: metric]` for numbers to be filled after benchmark runs.
