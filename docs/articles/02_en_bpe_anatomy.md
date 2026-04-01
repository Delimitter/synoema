# The Anatomy of BPE: Why Python Wastes 46% of Tokens

## How BPE Tokenization Works and What It Means for Language Design

---

> **Who this is for.** If you want to understand how ChatGPT "sees" your code and why the same program costs different amounts in different languages — read on. All terms explained in footnotes and the glossary at the end.

---

In the previous article, we established that inference cost grows quadratically with token count. The natural question: can we reduce token count without losing semantics?

To answer that, we need to understand how LLMs see code. Not as text — as a sequence of tokens. And between how a programmer sees `def factorial(n):` and how GPT-4 sees it, there's a chasm.

## How BPE Works

BPE (Byte Pair Encoding)[^bpe] is the algorithm that converts text into sequences of integers (tokens). It underlies all modern LLMs: GPT-4 uses the cl100k_base[^cl100k] vocabulary, Claude uses a modified BPE, and Llama uses SentencePiece[^sp] BPE.

[^bpe]: **BPE (Byte Pair Encoding)** — a text compression algorithm invented in 1994 and adapted for LLMs. The idea: find the most frequent pairs of characters in a huge text corpus and merge them into a new symbol. Repeat ~100,000 times. The result is a vocabulary of "subwords" that the model thinks in.

[^cl100k]: **cl100k_base** — the specific BPE token vocabulary used by GPT-4 and Claude. Contains ~100,000 tokens. Trained primarily on English internet text. GPT-4o uses a newer vocabulary called o200k_base with ~200,000 tokens.

[^sp]: **SentencePiece** — Google's alternative BPE implementation, used in Llama and other open models. Works at the Unicode character level instead of bytes, which is better for non-English languages.

The algorithm is simple:

1. Start with an alphabet of individual bytes (256 characters).
2. Find the most frequent pair of adjacent symbols in the corpus[^corpus].
3. Create a new symbol for that pair and add it to the vocabulary.
4. Repeat steps 2–3 until the desired vocabulary size (~100K).

[^corpus]: **Corpus** — the massive text dataset used to train the BPE vocabulary (and the LLM itself). Includes web pages, books, articles, GitHub code. Usually hundreds of billions of tokens. Code makes up only ~5–15% of a typical corpus — which is why BPE is optimized for English prose, not Python syntax.

The result is a vocabulary of ~100,000 "subwords" of variable length. Short, frequent words (`the`, `is`, `def`) encode as a single token. Rare words get split: `tokenization` → `token` + `ization`.

**The critical property:** BPE is trained on **natural language**, not code. So it's optimized for English prose, not Python syntax.

## The Misalignment Problem

The grammatical units of a programming language — operators, keywords, delimiters — **don't align** with BPE token boundaries. This creates two types of waste.

**Type 1: Redundant tokens on syntax.**

Take a simple Python function:

```python
def factorial(n):
    if n == 0:
        return 1
    return n * factorial(n - 1)
```

BPE (cl100k_base) splits this into **29 tokens**. Semantically significant: `factorial`, `n`, `0`, `1`, `*`, `-`. The remaining 23 tokens are syntactic overhead: `def`, spaces, `(`, `)`, `:`, `if`, `==`, `return` (twice), indentation, newlines.

The equivalent program in a minimal-syntax functional language:

```
fac 0 = 1
fac n = n * fac (n - 1)
```

**16 tokens.** Same semantics. 45% fewer.

**Type 2: Bridge tokens[^bridge].**

[^bridge]: **Bridge token** — a BPE token that spans the boundary between two grammatical symbols. For example, BPE might merge a space and keyword ` if` into one token. This creates problems for constrained decoding engines, which must "split" the token, distorting the model's probability distribution. More details in the third article.

Sometimes a single BPE token spans two grammatical symbols. For example, ` "name"` in JSON may become one token, even though grammatically it's space + quote + identifier + quote. This creates problems for constrained decoding[^cd].

[^cd]: **Constrained decoding** — technology that forbids invalid tokens at each generation step. Guarantees syntactically valid output. Covered in detail in the third article.

## Benchmark: 12 Programs, 3 Languages

I compared token counts for equivalent programs in three languages: Python, Haskell[^haskell], and an optimized language where every operator is exactly one BPE token.

[^haskell]: **Haskell** — a functional programming language with minimal syntax. Used as the "brevity benchmark" among existing languages. `fac 0 = 1` in Haskell and the optimized language look nearly identical, but the optimized language additionally accounts for BPE boundaries.

| Program | Optimized | Python | Haskell | Saving vs Python |
|---------|-----------|--------|---------|-----------------|
| Factorial | 16 | 29 | 16 | 45% |
| Map | 20 | 42 | 23 | 52% |
| QuickSort | 51 | 83 | 54 | 39% |
| FizzBuzz | 44 | 64 | 49 | 31% |
| Filter | 27 | 67 | 36 | 60% |
| Fibonacci | 26 | 46 | 26 | 43% |
| Sum | 16 | 33 | 19 | 52% |
| Length | 16 | 30 | 19 | 47% |
| Reverse | 18 | 35 | 18 | 49% |
| Compose | 38 | 75 | 39 | 49% |
| Maximum | 28 | 58 | 37 | 52% |
| Zip | 32 | 53 | 37 | 40% |
| **Total** | **332** | **615** | **373** | **46%** |

**The optimized language uses 46% fewer tokens than Python**, and 11% fewer than Haskell.

Given quadratic attention cost: 46% fewer tokens ≈ **71% less computation** in attention layers.

## Where the Savings Come From

### Function Definition

```python
# Python: 6 tokens of boilerplate
def add(x, y):
    return x + y

# Optimized: 0 boilerplate tokens
add x y = x + y
```

### Conditional

```python
# Python: 6 overhead tokens
if x > 0:
    return x
else:
    return -x

# Optimized: 3 tokens
? x > 0 -> x : -x
```

### Lists

```python
[1, 2, 3, 4, 5]  # 9 tokens (commas are waste)
[1 2 3 4 5]       # 7 tokens (no commas needed)
```

### Pattern Matching[^pm]

[^pm]: **Pattern matching** — defining a function by "examples." Instead of `if n == 0: return 1`, you write `fac 0 = 1` — literally: "factorial of zero is one." The compiler generates the check automatically. Shorter, clearer, and eliminates an entire class of errors.

```python
# Python: 29 tokens
def fac(n):
    if n == 0:
        return 1
    return n * fac(n - 1)

# Optimized: 16 tokens
fac 0 = 1
fac n = n * fac (n - 1)
```

## BPE-Aligned Grammar

The savings above aren't just "terse syntax." They're achieved through deliberate grammar design accounting for BPE tokenizer properties.

**The BPE-aligned grammar[^aligned] principle:** every language operator must be exactly one BPE token.

[^aligned]: **BPE-aligned grammar** — a language design principle where every operator, keyword, and delimiter encodes to exactly one BPE token. This means: no "wasted" tokens on syntax and no bridge tokens. Conventional languages don't account for BPE — they were created long before LLMs.

For the optimized language, all 33 operators were verified — each encodes to exactly 1 BPE token on cl100k_base (GPT-4/Claude) and o200k_base[^o200k] (GPT-4o):

[^o200k]: **o200k_base** — the newer BPE vocabulary used by GPT-4o. Contains ~200,000 tokens (twice cl100k_base). Better coverage of code and non-English languages, but same underlying principles.

```
Two chars, 1 token:    -> <- |> ++ >> == != <= >= && || ..
One char, 1 token:     ? : . = @ | \ _ , + - * / % < >
Delimiters, 1 token:   ( ) [ ]
```

This isn't coincidence — it's a **design constraint**. If an operator doesn't fit in one BPE token, it gets replaced by one that does.

## What This Means for LLMs

When an LLM generates code in the optimized language instead of Python, it generates 46% fewer tokens (faster, cheaper), spends 71% less on attention (larger codebases fit), creates no bridge tokens (cleaner constrained decoding), and can't "babble" (minimal syntax prevents bloat).

## What's Next

In the next article, we'll look at **constrained decoding** — the technology that guarantees 100% syntactic correctness. And we'll show why BPE-aligned grammar makes constrained decoding **free**.

---

*Second article in "Token Economics of Code." Benchmark: equivalent programs in three languages, tokenized via cl100k_base (GPT-4).*

---

## Glossary

| Term | Explanation |
|------|-----------|
| **BPE** | Byte Pair Encoding — algorithm splitting text into tokens by merging frequent character pairs |
| **cl100k_base** | GPT-4/Claude's BPE vocabulary with ~100K tokens |
| **o200k_base** | GPT-4o's newer BPE vocabulary with ~200K tokens |
| **SentencePiece** | Google's BPE alternative used in Llama and open models |
| **Corpus** | Massive text dataset for training BPE and LLMs (web, books, code) |
| **Bridge token** | BPE token spanning the boundary of two grammar symbols |
| **BPE-aligned grammar** | Grammar where every operator = exactly 1 BPE token |
| **Pattern matching** | Defining functions by examples: `fac 0 = 1` instead of `if n == 0` |
| **Constrained decoding** | Technology forbidding invalid tokens during generation |
| **Haskell** | Functional language with minimal syntax, brevity benchmark |
