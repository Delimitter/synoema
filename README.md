# Synoema

***The language of shared understanding***

**Version: 0.1.0-alpha.2** — alpha stage, syntax and APIs may change. See [versioning policy](docs/versioning.md).

**Synoema** [sy-NO-e-ma] — a BPE-aligned programming language for LLM code generation. Saves **15% tokens** vs Python on average (up to 52% on algorithmic tasks), compiles to native code via Cranelift JIT (**3x median speedup** over Python).

> **Status: research project.** Active research prototype — not yet production-ready. Contributions welcome.

## 30 Seconds

```bash
synoema eval "6 * 7"
# → 42
```

## Install

**Pre-built binary** (recommended) — download from [GitHub Releases](https://github.com/Delimitter/synoema/releases/latest) for macOS, Linux, or Windows, then run:
```bash
./synoema install   # copies to ~/.synoema/bin, adds to PATH
```

**cargo install** (requires Rust):
```bash
git clone https://github.com/Delimitter/synoema && cd synoema
cargo install --path lang/crates/synoema-repl
```

**MCP server** (no Rust needed):
```bash
npx synoema-mcp
```

Full installation guide with troubleshooting: [docs/install.md](docs/install.md)

## Quick Wins

### Run a program

```bash
synoema run examples/quicksort.sno    # [1 2 3 4 5 6 7 8 9]
synoema jit examples/factorial.sno    # 3628800 (native speed)
```

### Evaluate expressions in the terminal

```bash
synoema eval "[1..10] |> filter (\x -> x % 2 == 0) |> sum"
# → 30
```

### Scaffold a new project

```bash
synoema init myapp
cd myapp
synoema run src/main.sno    # Hello, myapp!
```

### Plug into your LLM toolchain (MCP)

```json
{
  "mcpServers": {
    "synoema": { "command": "npx", "args": ["synoema-mcp"] }
  }
}
```

Works with Claude Desktop, Cursor, and Zed. Tools: `eval`, `typecheck`, `run`. See [docs/mcp.md](docs/mcp.md).

### Use in VS Code

```bash
cd vscode-extension && ./install.sh
```

Syntax highlighting, run/JIT commands, eval selection:
- `Cmd+Shift+R` — run file
- `Cmd+Shift+J` — JIT compile

## Show Me The Code

```sno
-- Factorial (16 tokens vs Python's 29)
fac 0 = 1
fac n = n * fac (n - 1)

-- QuickSort with list comprehension
qsort [] = []
qsort (p:xs) = qsort lo ++ [p] ++ qsort hi
  lo = [x | x <- xs , x <= p]
  hi = [x | x <- xs , x > p]

-- Pipes, lambdas, ranges
result = [1..10] |> filter (\x -> x % 2 == 0) |> map (\x -> x * x) |> sum

-- Algebraic data types
Maybe a = Just a | None

-- Records with punning
point x y = {x, y}
dist {x, y} = x * x + y * y

-- Type classes
trait Show a
  show : a -> String

Color = Red | Green | Blue derive (Show, Eq)

-- Conditional: ? -> :
fizzbuzz n =
  ? n % 15 == 0 -> "FizzBuzz"
  : ? n % 3 == 0 -> "Fizz"
  : ? n % 5 == 0 -> "Buzz"
  : show n
```

No `def`. No `return`. No commas in lists. Every operator is a single BPE token.

## Why Synoema?

**Token efficiency** — 15% fewer tokens than Python on average, verified on 16 benchmark tasks (automated, tiktoken cl100k_base):

```
Task              Synoema  Python  Saving
─────────────────────────────────────────────
json_build            32      67     52%
pattern_match        136     225     40%
quicksort             77     124     38%
mergesort            117     179     35%
gcd                   26      35     26%
fibonacci             38      49     22%
factorial             25      32     22%
fizzbuzz              59      63      6%
─────────────────────────────────────────────
matrix_mult          212     152    -39%
string_ops            28      15    -87%
─────────────────────────────────────────────
AVERAGE (16 tasks)  79.1    92.9     15%
```

Synoema excels at recursive algorithms and pattern-heavy code. Python is more compact for string operations and imperative-style tasks.

**Native speed** — JIT-compiled via Cranelift (12 tasks, median of 5 runs):

```
Benchmark         Python     Synoema JIT    Speedup
────────────────────────────────────────────────────
fibonacci          144ms         5.1ms       28.2×
factorial           24ms         5.7ms        4.2×
gcd                 17ms         4.7ms        3.5×
collatz             18ms         5.6ms        3.1×
quicksort           17ms         6.2ms        2.7×
matrix_mult         16ms         7.7ms        2.1×
────────────────────────────────────────────────────
Median (12 tasks)                              3.0×
```

Fibonacci is an outlier — deep recursion where JIT's TCO shines. Typical speedup: 2–4×.

**Guaranteed correctness** — GBNF grammar for constrained decoding ensures 100% syntactically valid output from any LLM.

**Scientific foundation** — design grounded in peer-reviewed research on token efficiency, type-guided generation, and grammar-constrained decoding. See [docs/research/scientific_foundations.md](docs/research/scientific_foundations.md).

> Full benchmark methodology, all results, and how to reproduce: [docs/benchmarks.md](docs/benchmarks.md)

## Documentation

| Document | Description |
|----------|-------------|
| [Language Guide](docs/LANGUAGE.md) | Complete language reference with examples |
| [Benchmarks](docs/benchmarks.md) | Token savings, runtime, LLM generation benchmarks |
| [Contributing](CONTRIBUTING.md) | Build from source, architecture, how to contribute |
| [Installation](docs/install.md) | All install methods, MCP setup, troubleshooting |
| [LLM Reference](docs/llm/synoema.md) | Token-optimized quick reference for LLMs |
| [MCP Server](docs/mcp.md) | Claude Desktop / Cursor / Zed integration |
| [Formal Spec](docs/specs/language_reference.md) | EBNF grammar, type rules, operational semantics |
| [Scientific Foundations](docs/research/scientific_foundations.md) | 23 peer-reviewed sources behind the design |
| [Examples](lang/examples/) | 44 example programs — algorithms, data structures, patterns, tools |

## License

Apache-2.0 — see [LICENSE](LICENSE) for details. Some components use different licenses:

| Directory | License |
|-----------|---------|
| `lang/crates/synoema-codegen/` | BSL-1.1 → Apache-2.0 (after 36 months) |
| `tools/` | BSL-1.1 → Apache-2.0 (after 36 months) |
| `docs/`, `spec/` | CC-BY-SA-4.0 |
| `examples/` | MIT-0 (no attribution required) |
