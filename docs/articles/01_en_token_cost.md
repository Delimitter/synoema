# Why Every Token Costs More Than You Think

![Cover](images/cover_01.png)

## The Quadratic Price of Attention: How Context Length Is Killing Your AI Budget

---

> **Who this is for.** If you use ChatGPT, Claude, Copilot, or Cursor to write code, this article explains why the same tasks can cost 2–4× less. No technical background required — all terms are explained inline and in the glossary at the end.

---

When you ask Claude or GPT to write a sorting function, the model generates ~50 tokens[^token] per second. Each token costs fractions of a cent. Seems cheap.

But behind that simplicity lies an engineering reality most people overlook: the cost of each token grows **quadratically** with context length[^context]. If you're working with codebases spanning thousands of lines, this quadratic relationship transforms from a theoretical abstraction into a line item that can double your AI budget.

In this article, I'll show where this cost comes from, why inference — not training — is the dominant consumer of resources, and what can be done about it.

## Inference Consumes 90%+ of All Energy

There's a common misconception: the major cost of LLMs[^llm] is training. Training GPT-4 reportedly cost $50–100M. An impressive number.

But training is a one-time capital expense. Inference[^inference] is an ongoing operational cost that occurs with every request, every second, for every user.

According to AWS, inference consumes more than 90% of total energy in the LLM lifecycle. The AI inference market is valued at $106 billion in 2025, projected to exceed $250 billion by 2030 at a 19.2% compound annual growth rate.

Every token ChatGPT generates costs OpenAI approximately $0.00012. Sounds negligible. But at billions of daily requests, this adds up to hundreds of millions of dollars per year — and terajoules of electricity.

## The Quadratic Trap

Here's the key fact that changes everything.

In a standard transformer[^transformer] with self-attention[^attention], the computational cost of processing a sequence of *n* tokens is:

```
Cost(n) = O(n² · d)
```

where *d* is the model dimension. This is not a linear relationship. It's **quadratic**.

What this means in practice:

| Context | Relative attention cost |
|---------|----------------------|
| 1,000 tokens | 1× (baseline) |
| 2,000 tokens | 4× |
| 4,000 tokens | 16× |
| 8,000 tokens | 64× |
| 32,000 tokens | 1,024× |

Doubling the context increases attention cost **fourfold**, not twofold. This means reducing context by 50% saves not 50%, but **75%** of attention computation.

When a developer sends a 2,000-line Python file (~8,000 tokens) to an LLM for refactoring, the attention cost is 64× higher than for a simple 1,000-token question. And that's just one request.

## Real Money

Let's calculate for a typical team.

A team of 10 developers uses an AI assistant (Cursor, Copilot, Claude Code). Each makes an average of 100 requests per day. Average request context: 2,000 input tokens. Average response: 500 output tokens.

At Claude Sonnet 4 pricing ($3/M input, $15/M output):

```
Input:  10 × 100 × 2,000 = 2M tokens/day × $3/M  = $6/day
Output: 10 × 100 × 500   = 500K tokens/day × $15/M = $7.50/day
Total: ~$13.50/day = ~$405/month
```

Now imagine expressing the same programs with 46% fewer tokens (I'll show in the next article that this is achievable):

```
Input:  2M × 0.54 = 1.08M tokens/day × $3/M  = $3.24/day
Output: 500K × 0.54 = 270K tokens/day × $15/M = $4.05/day
Total: ~$7.29/day = ~$219/month
```

Savings: **$186/month** for 10 people, or **$2,200/year**.

For 100 developers: **$22,000/year**. For 1,000: **$220,000**. And this is a conservative estimate with a relatively affordable model and moderate workload.

## The Energy Dimension

Measurements on LLaMA-65B[^llama] (A100 GPUs[^gpu]) show energy consumption in the range of 3–4 joules per output token. On modern H100s with optimized inference engines like vLLM[^vllm], efficiency has improved roughly 10×, down to ~0.39 J per token. But usage scale has grown even faster.

ChatGPT processes an estimated one billion requests daily. At an average response of 500 tokens:

```
1B requests × 500 tokens × 0.39 J ≈ 195 GJ/day ≈ 54,000 kWh/day
```

That's the energy consumption of a small town — every single day. Reducing token count isn't just about saving money. It's a direct reduction in energy consumption and carbon footprint.

## The Babbling Problem

The study "Towards Green AI" (2026) found that 3 out of 10 tested models exhibit "babbling" behavior — generating significantly more text than necessary. Suppressing this yielded energy savings of 44% to 89%.

But what if the language the LLM writes code in is designed so that "babbling" is physically impossible?

Python code is inherently verbose. `def`, `return`, `if/elif/else`, commas in lists — all syntactic overhead[^overhead] that consumes tokens without carrying semantic information.

## Three Optimization Levers

**Lever 1: Representation compression.** Express the same program with fewer tokens. This isn't obfuscation — it's grammar design optimized for BPE tokenizers[^bpe]. Potential: 35–50%.

**Lever 2: Constrained decoding[^constrained].** Prevent the model from generating syntactically invalid code. Every error = retry = double token spend.

**Lever 3: Type guarantees.** Type errors account for 33.6% of all failures in LLM-generated code. Type-guided generation[^typeguided] reduces them by 74.8%.

Combining all three levers can yield 60–80% cumulative savings in tokens, money, energy, and time.

## What's Next

In the next article, we'll examine **how BPE tokenization actually works** and why Python syntax wastes 46% of tokens on structural noise.

---

*First article in the "Token Economics of Code" series. Sources: TokenPowerBench (arxiv:2512.03024), "Towards Green AI" (EuroMLSys 2026), Mündler et al. (PLDI 2025), MarketsandMarkets.*

---

## Footnotes

[^token]: **Token** — the smallest unit of text an LLM processes. Not a letter, not a word, but a "chunk" of text 1–15 characters long. The word "hello" is 1 token; the code `def factorial(n):` is 6 tokens. The model doesn't see characters — it sees a sequence of tokens.

[^context]: **Context (context window)** — everything the model "sees" at once: your question, previous messages, attached files. Measured in tokens. GPT-4 has a context of up to 128K tokens, Claude up to 200K. The longer the context, the more computation the model needs.

[^llm]: **LLM (Large Language Model)** — a neural network trained on massive amounts of text that can generate text, code, and answer questions. Examples: GPT-4, Claude, Llama, Gemini.

[^inference]: **Inference** — the process of using an already-trained model to generate responses. When you type a prompt into ChatGPT and get an answer, that's inference. Unlike training (which happens once), inference happens billions of times per day.

[^transformer]: **Transformer** — the neural network architecture underlying all modern LLMs. Invented at Google in 2017 ("Attention Is All You Need" paper). Its key feature is the "attention" mechanism, which lets the model consider relationships between any words in the text, even distant ones.

[^attention]: **Self-attention** — a mechanism where every token "looks at" every other token in the context to understand their relationships. This gives transformers their power — but also creates quadratic cost: if there are *n* tokens, there are *n × n* pairs to compare.

[^llama]: **LLaMA** — a family of open-source language models from Meta (Facebook). Available for download and self-hosted deployment, unlike GPT-4.

[^gpu]: **GPU (Graphics Processing Unit)** — originally a graphics card, now used for AI computation. NVIDIA A100 and H100 are specialized GPUs for LLM inference and training. A single H100 costs ~$30–40K and draws 700 watts.

[^vllm]: **vLLM** — an open-source engine for fast LLM serving. Optimizes GPU memory usage through PagedAttention, enabling more simultaneous requests.

[^overhead]: **Syntactic overhead** — parts of code required by the language's syntax but carrying no meaning. For example, Python's `def` before a function definition and `return` before a return value are mandatory but contain no information about *what* the function does.

[^bpe]: **BPE (Byte Pair Encoding)** — the algorithm that splits text into tokens. Used in all modern LLMs. Finds the most frequent pairs of characters in a huge text corpus and merges them into new "subwords." Result: a vocabulary of ~100,000 tokens. Covered in detail in the second article.

[^constrained]: **Constrained decoding** — a technology that forbids the model from choosing invalid tokens at each generation step. If the model is generating JSON, it ensures brackets are closed and commas are in the right places. The same can be done for any language with a formal grammar.

[^typeguided]: **Type-guided generation** — an extension of constrained decoding where the model is additionally prevented from generating code with type errors. A second layer of guarantees on top of syntactic ones.

## Glossary

| Term | Explanation |
|------|-----------|
| **Token** | Smallest text unit for an LLM. Roughly ¾ of a word or 3–4 characters |
| **LLM** | Large Language Model — neural network that generates text/code (GPT-4, Claude, Llama) |
| **Inference** | Generating a response from a trained model. Happens with every request |
| **Context** | Everything the model "sees" — prompt, chat history, files. Measured in tokens |
| **Transformer** | Neural network architecture underlying all LLMs. Uses attention mechanism |
| **Self-attention** | Mechanism where every token considers all others. Cost: O(n²) |
| **BPE** | Byte Pair Encoding — algorithm that splits text into tokens |
| **Constrained decoding** | Technology forbidding invalid tokens during generation |
| **GPU** | Graphics card for AI computation. NVIDIA H100 is standard for LLM inference |
| **vLLM** | Open-source engine for fast LLM serving |
| **Overhead** | Parts of code/computation carrying no useful payload |
