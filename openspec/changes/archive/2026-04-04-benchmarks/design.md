# Design: Benchmark Suite

## Architecture

```
benchmarks/
├── runner/                    Rust CLI (orchestrator)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs            CLI args, phase dispatch, telemetry
│       ├── phases/
│       │   ├── mod.rs
│       │   ├── tokens.rs      Phase A: invoke token_count.py per task
│       │   ├── runtime.rs     Phase B: compile/run each language, measure
│       │   └── llm.rs         Phase C: invoke llm_generate.py per (task,lang,model)
│       ├── telemetry.rs       Live terminal output + progress
│       └── report.rs          Generate summary.txt, details.txt, raw.json
├── scripts/
│   ├── token_count.py         tiktoken cl100k_base counting
│   └── llm_generate.py        OpenRouter API (OpenAI-compatible)
├── tasks/
│   ├── factorial/
│   │   ├── factorial.sno
│   │   ├── factorial.py
│   │   ├── factorial.js
│   │   ├── factorial.ts
│   │   ├── factorial.cpp
│   │   └── expected_output.txt
│   ├── fibonacci/
│   │   └── ...
│   └── ... (16 task directories)
└── results/                   gitignored, populated by runs
```

## Rust CLI Design

```
cargo run -- run --all --openrouter-key KEY
cargo run -- run --phases token,runtime
cargo run -- run --phases llm --models gpt-4o,qwen3-coder-next
cargo run -- run --phases llm --tier weak
```

### CLI Arguments
- `--all` — run all phases (A + B + C)
- `--phases token,runtime,llm` — select phases
- `--openrouter-key KEY` — API key for Phase C
- `--models model1,model2` — filter models
- `--tier frontier|mid|weak` — filter by tier
- `--tasks task1,task2` — filter tasks
- `--repeats N` — override repeat count (default 5)

### Phase Orchestration
1. Phase A (tokens): parallel per task — calls `python3 scripts/token_count.py <dir>`
2. Phase B (runtime): sequential per task — calls language-specific compiler/runner
3. Phase C (llm): sequential per (model × task × language) — calls `python3 scripts/llm_generate.py`
4. Report generation: reads all JSON intermediates, writes summary.txt + details.txt + raw.json

## Python Scripts

### token_count.py
- Input: task directory path
- Reads all language files, counts tokens via tiktoken (cl100k_base)
- Output: JSON to stdout `{"synoema": N, "python": N, ...}`

### llm_generate.py
- Input: `--model MODEL --language LANG --task TASK_DIR --key KEY --context CONTEXT_FILE`
- Sends prompt to OpenRouter API (OpenAI-compatible endpoint)
- For Synoema: includes docs/llm/synoema.md as context
- Validates: compile/parse + run + compare output
- Output: JSON to stdout `{"syntax_ok": bool, "correct": bool, "tokens_in": N, "tokens_out": N, "code": "..."}`

## Runtime Measurement (Phase B)

Per language execution:
- **Synoema**: `cargo run -p synoema-repl -- jit <file>`
- **Python**: `python3 <file>`
- **JavaScript**: `node <file>`
- **TypeScript**: `npx ts-node <file>` (or `npx tsx <file>`)
- **C++**: `g++ -O2 -o /tmp/bench <file> && /tmp/bench`

Timing: Rust `std::time::Instant` around subprocess. Memory: `/usr/bin/time -l` on macOS.

## Telemetry (live terminal output)

Progress bar + latest result table, updated after each measurement.
Uses ANSI escape codes (cursor up, clear line) for in-place updates.
Final output = summary.txt content printed to terminal.

## Key Decisions
- tiktoken (not manual counter) — accurate cl100k_base tokenization
- OpenRouter (not direct APIs) — single endpoint for all non-Claude models
- Rust orchestrator (not Python) — neutral language, precise timing
- Python for token counting + LLM API — tiktoken and openai SDK are Python-native
- results/ gitignored — raw data not committed, only docs/benchmarks.md updated manually
