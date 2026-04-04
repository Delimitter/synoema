# Design: Ollama Integration

## Architecture

```
benchmarks/runner/src/
├── main.rs              + --ollama, --ollama-model flags
├── phases/
│   ├── llm.rs           + ollama_available(), ensure_model(), run_ollama() entry point
│   └── ...
└── ...

benchmarks/scripts/
└── llm_generate.py      + --base-url flag for ollama endpoint
```

## Detection Module (in llm.rs)

```rust
pub fn ollama_available() -> bool {
    Command::new("ollama").arg("--version").output()
        .map(|o| o.status.success()).unwrap_or(false)
}

pub fn ensure_model(model: &str, verbose: bool) -> Result<(), String> {
    // ollama list → check if model present → ollama pull if not
}
```

## CLI Changes (main.rs)

```rust
// Inside Run variant:
#[arg(long)]
ollama: bool,

#[arg(long, default_value = "qwen3:8b")]
ollama_model: String,
```

## llm_generate.py Changes

Add `--base-url` argument (default: OpenRouter URL). When ollama, pass `http://localhost:11434/v1`.

## Test Strategy

Tests in `benchmarks/runner/src/phases/llm.rs` (inline `#[cfg(test)]` module):
- `test_ollama_detection` — just checks the function runs without panic
- `test_ensure_model` — `#[ignore]`, requires ollama, pulls qwen3:8b
- `test_ollama_single_task` — `#[ignore]`, full round-trip: factorial/synoema via ollama
