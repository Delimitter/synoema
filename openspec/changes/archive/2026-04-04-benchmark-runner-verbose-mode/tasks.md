# Tasks: Benchmark Runner Verbose Mode

## 1. Add --verbose flag to CLI
- [x] 1.1 Add `verbose: bool` field with `#[arg(long, short = 'v')]` to `Command::Run`
- [x] 1.2 Extract `verbose` in match arm and pass to phase functions

## 2. Update token phase (phases/tokens.rs)
- [x] 2.1 Add `verbose: bool` parameter to `tokens::run()`
- [x] 2.2 In verbose mode: print python command before executing, print stderr if non-empty

## 3. Update runtime phase (phases/runtime.rs)
- [x] 3.1 Add `verbose: bool` parameter to `runtime::run()`
- [x] 3.2 In verbose mode: print command for each language, print each warm-up timing, print each measured run timing
- [x] 3.3 In verbose mode: print C++ compile command

## 4. Update LLM phase (phases/llm.rs)
- [x] 4.1 Add `verbose: bool` parameter to `llm::run()`
- [x] 4.2 In verbose mode: print model + prompt details, per-attempt results

## 5. Verify
- [x] 5.1 `cargo build --manifest-path benchmarks/runner/Cargo.toml` succeeds
- [x] 5.2 `--help` shows `--verbose` / `-v` flag
