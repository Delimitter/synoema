# Synoema

***The language of shared understanding***

**Version: 0.1.0-alpha.1** — alpha stage, syntax and APIs may change. See [versioning policy](docs/versioning.md).

**Synoema** [sy-NO-e-ma] — a BPE-aligned programming language for LLM code generation. Saves **46% tokens** vs Python, compiles to native code via Cranelift JIT at **4.4x Python speed**.

> **Status: research project.** Active research prototype — not yet production-ready. Contributions welcome.

## 30 Seconds

```bash
synoema eval "6 * 7"
# → 42
```

## Install

**cargo install** (requires Rust):
```bash
git clone https://github.com/Delimitter/synoema && cd synoema
cargo install --path lang/crates/synoema-repl
```

**Pre-built binary** — download from [GitHub Releases](https://github.com/Delimitter/synoema/releases/latest) for macOS, Linux, or Windows.

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

**Token efficiency** — 46% fewer tokens than Python, verified on 12 programs:

```
Program              Synoema  Python  Saving
─────────────────────────────────────────────
Factorial               16      29     45%
QuickSort               51      83     39%
FizzBuzz                44      64     31%
Filter                  27      67     60%
─────────────────────────────────────────────
TOTAL (12 programs)    332     615     46%
```

Due to quadratic attention cost, **46% fewer tokens ≈ 71% less attention compute.**

**Native speed** — JIT-compiled via Cranelift:

```
Benchmark           Python    Synoema JIT    Speedup
────────────────────────────────────────────────────
fib(30)              277ms       47ms          5.9×
collatz (10K)        505ms       90ms          5.6×
────────────────────────────────────────────────────
Average                                        4.4×
```

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
| [Examples](lang/examples/) | 24 example programs from factorial to HTTP server |

## License

MIT
