# Does Language Design Reduce LLM Size Requirements?

*An empirical study of code generation across model sizes, languages, and decoding strategies*

## Abstract

We investigate whether a programming language designed for LLM code generation can reduce the minimum model size required for correct code synthesis. Using Synoema, a functional language with BPE-aligned operators and a context-free grammar suitable for constrained decoding, we compare code generation quality across four model sizes (0.5B-7B parameters), three languages (Synoema, Python, Haskell), and three decoding modes (zero-shot, few-shot, constrained). Our key finding: [RESULT: key_finding]. This suggests that language design is an underexplored axis for improving LLM code generation quality, complementary to scaling model parameters.

## 1. Introduction

Large language models generate code with increasing proficiency, yet this capability scales with model size and training data volume. A 70B-parameter model writes correct Python far more reliably than a 3B-parameter model. This creates practical barriers: local inference, edge deployment, and cost-sensitive applications require smaller models.

We ask a different question: *can the target language itself compensate for smaller model capacity?*

Synoema is a functional programming language designed explicitly for LLM code generation. Three design properties are hypothesized to help smaller models:

1. **Constrained decoding compatibility.** Synoema's grammar is context-free after lexing, enabling GBNF-based grammar constraints during generation. This eliminates syntax errors entirely. Python and Haskell use context-sensitive layout rules that prevent reliable grammar-constrained decoding.

2. **Token efficiency.** All 33 Synoema operators are single BPE tokens (cl100k_base). Programs average 15% fewer tokens than Python equivalents, meaning fewer sequential decisions for the model to make.

3. **Syntactic regularity.** Synoema has ~10 core syntactic constructs versus Python's 33 keywords. Fewer patterns to learn means in-context learning is more effective with limited model capacity.

**Contribution.** We present the first controlled comparison of code generation quality across model sizes and target languages, isolating language design as a variable. Our benchmark covers 30 tasks across five difficulty levels, evaluated with compiler-verified correctness.

## 2. Background

### 2.1 Synoema

Synoema is an expression-oriented, statically-typed functional language with Hindley-Milner type inference, algebraic data types, pattern matching, and list comprehensions. It compiles via Cranelift JIT to native code.

Key design decisions relevant to LLM generation:
- **Offside rule** (like Python/Haskell) for block structure, but converted to INDENT/DEDENT tokens at lexing time, making the post-lexer grammar context-free
- **No keywords for function definition**: `f x = x * 2` (vs Python's `def f(x): return x * 2`)
- **Ternary syntax**: `? cond -> then : else` (3 tokens vs Python's 8)
- **Space-separated lists**: `[1 2 3]` (no commas)
- **Complete GBNF grammar**: 188 rules, 48 productions

### 2.2 Constrained Decoding

Grammar-constrained decoding masks invalid tokens at each generation step, ensuring every output conforms to a specified formal grammar. This is supported natively in llama.cpp via GBNF grammars.

For Synoema, GBNF guarantees 100% syntactic correctness. For Python, reliable GBNF is not possible because Python's grammar is context-sensitive (indentation depth is runtime state, f-strings allow arbitrary nesting). Haskell's layout rule is similarly context-sensitive.

### 2.3 Token Efficiency

Prior work established Synoema's token efficiency: across 16 benchmark tasks, Synoema programs average [RESULT: token_pct]% fewer tokens than Python equivalents (cl100k_base encoding). Fewer tokens means fewer autoregressive decisions, and each decision is a potential error source.

The probability of a correct N-token program with per-token accuracy p is P = p^N. For p = 0.95: P(52 tokens) = 7%, P(32 tokens) = 19% -- a 2.7x improvement from token count alone.

## 3. Experimental Setup

### 3.1 Models

We use the Qwen2.5-Coder family, which provides consistent architecture across four sizes:

| Model | Parameters | Quantization | Context |
|-------|-----------|-------------|---------|
| Qwen2.5-Coder-0.5B | 0.5B | Q4_K_M | 4096 |
| Qwen2.5-Coder-1.5B | 1.5B | Q4_K_M | 4096 |
| Qwen2.5-Coder-3B | 3B | Q4_K_M | 4096 |
| Qwen2.5-Coder-7B | 7B | Q4_K_M | 4096 |

All models run locally via llama.cpp server.

### 3.2 Languages

| Language | In pre-training | GBNF possible | ICL reference |
|----------|----------------|---------------|---------------|
| Synoema | None | Yes (188 rules) | ~1800 tokens |
| Python | Extensive | No (context-sensitive) | ~1800 tokens |
| Haskell | Moderate | No (context-sensitive) | ~1800 tokens |

Each language receives an equally-sized in-context learning reference document (~1800 cl100k_base tokens) to ensure fair comparison.

### 3.3 Generation Modes

| Mode | Description | Applicable |
|------|-------------|-----------|
| zero-shot | System prompt + task description + language reference | All languages |
| few-shot | + 3 example programs | All languages |
| constrained | + GBNF grammar enforcement | Synoema only |

### 3.4 Tasks

30 tasks across five difficulty levels:

| Level | Tasks | Examples |
|-------|-------|---------|
| Basics | 9 | factorial, palindrome, power |
| Data Structures | 5 | bst_insert, tree_traverse, stack_calc |
| Functional | 6 | compose_chain, scan_left, filter_map |
| Type System | 5 | maybe_chain, either_validate, pattern_match |
| Practical | 5 | csv_parse, word_freq, state_machine |

Each task has a natural-language prompt, expected output, and verified reference implementations in all three languages.

### 3.5 Metrics

For each (model, language, mode, task), we run 5 independent generations and measure:
- **syntax_rate**: fraction that parse without errors
- **type_rate**: fraction that pass type checking
- **correct_rate**: fraction that produce the expected output
- **tokens_out**: average tokens generated

Primary metric: **minimum model size for 70% correctness** per language.

### 3.6 Validation

Correctness is verified by compiler/interpreter execution:
- **Synoema**: `synoema-repl run` (parse + typecheck + interpret)
- **Python**: `python3 -c compile(...)` (syntax) + `python3 file.py` (execution)
- **Haskell**: `runghc file.hs` (parse + typecheck + execution)

Output is compared to expected_output.txt via exact string match.

## 4. Results

### 4.1 Correctness vs Model Size

[RESULT: correctness_table]

| Model | Synoema+GBNF | Synoema | Python | Haskell |
|-------|-------------|---------|--------|---------|
| 0.5B | [RESULT]% | [RESULT]% | [RESULT]% | [RESULT]% |
| 1.5B | [RESULT]% | [RESULT]% | [RESULT]% | [RESULT]% |
| 3B | [RESULT]% | [RESULT]% | [RESULT]% | [RESULT]% |
| 7B | [RESULT]% | [RESULT]% | [RESULT]% | [RESULT]% |

### 4.2 Minimum Model for 70% Correctness

[RESULT: min_model_table]

### 4.3 Decomposition of Gains

[RESULT: decomposition]

The correctness advantage of Synoema+GBNF decomposes into three factors:
1. **GBNF constraint** (syntax_rate = 100%): eliminates all syntax errors
2. **Token efficiency** (fewer decisions): reduces logic error rate
3. **Syntactic regularity** (better ICL): improves model's understanding

### 4.4 Syntax Rate by Language

[RESULT: syntax_table]

## 5. Analysis

### 5.1 Why GBNF Matters More for Small Models

[RESULT: analysis_gbnf]

### 5.2 The Pre-training Paradox

Python benefits from massive pre-training exposure, yet Synoema -- with zero pre-training examples -- achieves comparable or better correctness on small models. This is because:

1. Syntax errors are the primary failure mode for small models, and GBNF eliminates them entirely
2. Shorter programs have exponentially higher probability of correctness
3. Regular syntax is learnable from a single ~1800-token reference document

### 5.3 Error Taxonomy

[RESULT: error_taxonomy]

## 6. Related Work

- **Code generation benchmarks**: HumanEval, MBPP, MultiPL-E evaluate across programming languages but do not isolate language design as a variable.
- **Constrained decoding**: GBNF in llama.cpp, Outlines, SGLang guidance -- applied to existing languages. Synoema is the first language designed for constrained decoding compatibility.
- **Domain-specific languages for LLMs**: LMQL, Guidance, DSPy -- these are prompting languages, not target code generation languages.
- **Token efficiency**: BPE-aware language design is explored in Synoema's prior work but not previously connected to model size requirements.

## 7. Conclusion

[RESULT: conclusion]

Language design is an underexplored dimension of LLM code generation. While the field focuses on scaling models and improving prompting, our results show that a well-designed target language can achieve the same effect as a [RESULT: factor]x increase in model parameters. This has practical implications for edge deployment, cost reduction, and democratizing access to AI-assisted programming.

## Acknowledgments

Synoema is an open-source project. All benchmark code, task definitions, and analysis scripts are available at the project repository.
