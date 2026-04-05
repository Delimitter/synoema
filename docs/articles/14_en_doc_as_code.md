# Executable Documentation: When Your Comments Become Tests

![Cover](images/cover_14.png)

## How Synoema Stores Documentation State in Code — and Why Stale Docs Are a Solved Problem

---

> **Who this is for.** If you've ever found an outdated docstring that claimed a function returned a string when it actually returns a list, this article is for you. Whether you maintain a library, write code with AI assistants, or just want documentation that can't lie — read on.

---

Documentation lies. Not intentionally, but inevitably. A developer changes a function's behavior, forgets to update the docstring, and now the documentation describes code that no longer exists. This drift is not a personal failing — it's a structural problem. Traditional programming languages treat documentation as metadata: a passive comment attached to code, never executed, never verified.

What if the language itself made stale documentation structurally impossible?

In this article — Part 14 of *Token Economics of Code* — I'll describe a paradigm where documentation is stored as executable state directly in the AST[^ast], verified on every test run, and consumed by both humans and LLMs from a single source of truth.

## The Scale of the Problem

The disconnect between documentation and code is well-documented (ironically). A 2023 study by Wen et al. found that 25.5% of Python docstrings in popular open-source projects are inconsistent with their corresponding function signatures. One in four.

The cost isn't just confusion. When an LLM reads a stale docstring to understand your codebase, it generates code based on incorrect context. That code fails. The failure triggers a retry. The retry consumes tokens. The tokens cost money and energy. Documentation debt becomes inference debt.

Three dominant approaches exist today. None solve the problem:

| Approach | Language | Verification | Drift risk |
|----------|----------|-------------|------------|
| **Docstrings** | Python | `doctest` (opt-in, fragile) | High — separate from tests |
| **JSDoc / TSDoc** | JS / TS | None — comments only | Very high |
| **Haddock** | Haskell | None — rendered to HTML | Moderate |

Python's `doctest` module comes closest, but it has a fundamental limitation: it compares string representations of output, not semantic values. A change in `__repr__` breaks every doctest. And doctest extraction relies on regex-level parsing, not the language's own AST.

## The Paradigm: Documentation as AST State

Synoema takes a different approach. Documentation is a **first-class syntactic element** — not a comment convention, but a token type recognized by the lexer, preserved in the AST, and consumed by the compiler toolchain.

The syntax uses triple-dash `---`:

```synoema
--- Compute factorial.
--- example: fact 5 == 120
fact 0 = 1
fact n = n * fact (n - 1)
```

Three things happen when the parser encounters `---`:

1. The lexer emits a `Token::DocComment(String)` — distinct from `--` (regular comment, stripped during tokenization)
2. The parser collects consecutive doc lines and attaches them to the next declaration as `doc: Vec<String>` in the AST
3. Lines starting with `example:` are flagged as executable assertions

This is not a wrapper around regular comments. It's a distinct token class, occupying exactly 1 BPE token[^bpe] in the `cl100k_base` vocabulary. Regular comments (`--`) are invisible to the AST. Doc comments (`---`) persist through the entire compilation pipeline.

Here is the key difference from traditional approaches:

```
Traditional:  Source → [strip comments] → AST → Compile
Synoema:      Source → AST (with doc: Vec<String>) → Compile + Test + Doc
```

Documentation is not stripped. It travels with the code.

## How It Works: From Lexer to Test Runner

Let me trace the full pipeline for a single doctest.

**Step 1: Lexing.** The scanner encounters `---` and calls `scan_doc_comment()`:

```
Input:  "--- example: fact 5 == 120\n"
Output: Token::DocComment("example: fact 5 == 120")
```

The text after `---` is captured verbatim, with leading whitespace trimmed.

**Step 2: Parsing.** The parser's `collect_doc_comments()` method gathers consecutive `DocComment` tokens into a vector. When it hits a function declaration, it attaches the vector:

```rust
Decl::Func {
    name: "fact",
    equations: [...],
    doc: ["Compute factorial.", "example: fact 5 == 120"],
    span: ...,
}
```

Both the human-readable description ("Compute factorial") and the executable assertion ("example: fact 5 == 120") live in the same `Vec<String>`. No separate metadata structure. No JSON sidecar. One field.

**Step 3: Test extraction.** When you run `synoema test`, the `extract_doctests()` function walks every declaration, finds lines starting with `example:`, and splits them:

```
"example: fact 5 == 120"
         ^^^^^^    ^^^
         expr      expected
```

The split respects bracket nesting — `example: head [1 2 3] == 1` correctly identifies `head [1 2 3]` as the expression and `1` as the expected value.

**Step 4: Execution.** Each doctest is evaluated by appending it to the full module source:

```synoema
-- Original source loaded here --
__doctest_val = fact 5
```

The result is compared against the expected value (also evaluated in the same context). If they match — pass. If not — fail with a diagnostic showing the expression, expected value, and actual value.

This means doctests have access to every definition in the file. They run in the real evaluation environment, not a sandboxed mock. If the function changes behavior, the doctest catches it.

## Three Testing Tiers, One Pipeline

Synoema unifies three kinds of verification into a single `synoema test` command:

**Tier 1: Doctests** — inline assertions in doc comments.

```synoema
--- Reverse a list.
--- example: reverse [1 2 3] == [3 2 1]
reverse [] = []
reverse (x:xs) = reverse xs ++ [x]
```

**Tier 2: Unit tests** — named boolean assertions using the `test` keyword.

```synoema
test "fact base" = fact 0 == 1
test "fact 10" = fact 10 == 3628800
test "sort then reverse" = reverse (qsort [3 1 2]) == [3 2 1]
```

**Tier 3: Property tests** — generative testing with the `prop` keyword.

```synoema
test "reverse involution" = prop xs -> reverse (reverse xs) == xs
test "sort idempotent" = prop xs -> qsort (qsort xs) == qsort xs
test "fact positive" = prop n -> fact n >= 1 when n >= 0 && n <= 10
```

Property tests use Hindley-Milner[^hm] type inference to determine what values to generate. The variable `xs` in `prop xs -> reverse (reverse xs) == xs` is inferred as `List a` — so the test runner generates random lists. The variable `n` in `prop n -> fact n >= 1` is inferred as `Int` — so it generates random integers. No manual type annotations required.

The `when` clause filters generated values: `when n >= 0 && n <= 10` discards any `n` outside that range before evaluating the property. 100 valid trials per property, deterministic seed for reproducibility.

All three tiers run together:

```bash
$ synoema test examples/testing.sno

  testing.sno
    doctests:    4 passed, 0 failed
    unit tests:  4 passed, 0 failed
    properties:  5 passed, 0 failed (500 trials)

  Total: 13 passed, 0 failed
```

## Documentation Generation: Same Source, Different Output

The same `doc: Vec<String>` that drives testing also drives documentation generation. The `synoema doc` command reads the AST and renders it — without re-parsing, without a separate doc format, without Markdown source files.

**Markdown output** (`synoema doc --format md`):

Interleaves doc lines as prose and declarations as code blocks. Lines starting with `example:` are rendered as highlighted code snippets. Metadata lines (`guide:`, `order:`, `requires:`) control page title, ordering, and dependency tracking — but are invisible in the rendered output.

**JSON output** (`synoema doc --format json`):

Exports structured metadata for tooling:

```json
{
  "file": "examples/testing.sno",
  "functions": [
    {
      "name": "fact",
      "doc": ["Compute factorial.", "example: fact 5 == 120"],
      "line": 7
    }
  ]
}
```

This JSON is consumed by the MCP server[^mcp], which exposes Synoema documentation to LLM agents. The documentation that LLMs read is the same documentation that tests verify. There is no gap.

## Why LLMs Care

When an LLM generates or modifies Synoema code, it reads doc comments as part of the source context. Those comments are guaranteed to be accurate — because if they weren't, `synoema test` would have failed.

This creates a feedback loop:

```
LLM reads doc → generates code → code changes behavior →
  synoema test catches stale docs → developer updates docs →
    LLM reads updated docs → ...
```

In traditional languages, the loop has a silent gap: nothing catches stale docs. The LLM operates on incorrect context, generates incorrect code, and the developer blames the LLM rather than the documentation.

There's a second benefit specific to token economics. Doc comments in Synoema use `---` (1 BPE token) instead of Python's `"""..."""` (at least 2 tokens for delimiters) or JSDoc's `/** ... */` (3+ tokens). Each `example:` line is 1 token for the keyword. The documentation syntax itself is token-efficient — consistent with the language's design principle of minimizing BPE token count.

## Side-by-Side: Python vs Synoema

Let's compare equivalent documented, tested code:

**Python (32 tokens):**

```python
def fact(n):
    """Compute factorial.

    >>> fact(5)
    120
    """
    if n == 0:
        return 1
    return n * fact(n - 1)
```

**Synoema (14 tokens):**

```synoema
--- Compute factorial.
--- example: fact 5 == 120
fact 0 = 1
fact n = n * fact (n - 1)
```

Same function. Same documentation. Same executable test. 56% fewer tokens. But the meaningful difference isn't token count — it's that Python's doctest compares string output (`"120"`) while Synoema's compares evaluated values (`120 == 120`). Change `fact` to return a float, and Python's doctest breaks on `"120.0"`. Synoema's doesn't.

## Try It

Install and run doctests on the example suite:

```bash
# Install Synoema
cargo run -p synoema-repl -- install

# Run all tests (doctests + unit + property)
synoema test examples/

# Run tests for a specific file
synoema test examples/testing.sno

# Generate documentation
synoema doc examples/testing.sno
synoema doc --format json examples/testing.sno
```

Write your own documented function:

```synoema
--- Double every element in a list.
--- example: double_all [1 2 3] == [2 4 6]
double_all xs = [x * 2 | x <- xs]
```

Save it as `my_funcs.sno` and run `synoema test my_funcs.sno`. The example assertion becomes a test. The description becomes documentation. One source, two outputs, zero drift.

## What's Next

In the next article, we'll explore the **future of code generation** — how compilation, type inference, and executable documentation combine into an agentic pipeline where LLMs don't just write code, but verify it.

---

*Part 14 of "Token Economics of Code" by @andbubnov. Synoema is open-source: [github.com/Delimitter/synoema](https://github.com/Delimitter/synoema).*

---

## Footnotes

[^ast]: **AST (Abstract Syntax Tree)** — the tree structure a compiler builds from your source code. Each node represents a language construct: a function, an expression, a type. In most languages, comments are discarded before the AST is built. In Synoema, doc comments survive into the AST as data attached to declarations.

[^bpe]: **BPE (Byte Pair Encoding)** — the tokenization algorithm used by all major LLMs. It splits text into "tokens" — chunks of 1-15 characters. The fewer tokens a program requires, the less it costs to process. Covered in detail in [Part 2 of this series](#).

[^hm]: **Hindley-Milner** — a type inference algorithm that determines the types of all expressions without requiring explicit type annotations. Synoema uses it to infer parameter types for property tests, enabling automatic test data generation. Covered in [Part 5 of this series](#).

[^mcp]: **MCP (Model Context Protocol)** — a protocol for connecting LLM agents to external tools and data sources. Synoema's MCP server exposes language features (evaluation, documentation, type checking) directly to AI assistants.

## Glossary

| Term | Explanation |
|------|-----------|
| **AST** | Abstract Syntax Tree — the parsed structure of source code |
| **BPE** | Byte Pair Encoding — how LLMs split text into tokens |
| **Doctest** | An executable example embedded in documentation |
| **Doc comment** | A `---` line in Synoema that persists in the AST |
| **Hindley-Milner** | Type inference algorithm — determines types without annotations |
| **MCP** | Model Context Protocol — connects LLM agents to external tools |
| **Property test** | A test that verifies a property holds for random inputs |
| **Token** | Smallest text unit for an LLM, roughly 3-4 characters |
