# Proposal: env-args

## Problem Statement

Synoema programs cannot access environment variables or command-line arguments from the interpreter. This blocks LLM-generated programs that need to read config from the environment (API keys, paths) or process CLI inputs (scripts, automation). Both are standard capabilities expected in any scripting language.

## Scope

### Feature 1: Environment variables (`env`, `env_or`)

- `env : String -> String` — reads env var, returns `""` if not set
- `env_or : String -> String -> String` — reads env var, returns default if not set
- Interpreter-only (no JIT — interpreter-first rule)
- Implemented as builtins in `eval.rs`

### Feature 2: CLI arguments (`args`)

- `args : [String]` — list of arguments passed after `--` separator
- Injected into top-level env at program start
- REPL parses `--` separator: `synoema run file.sno -- a b c`
- Interpreter-only

## Success Criteria

- `env "HOME"` → non-empty string (in interpreter)
- `env "NONEXISTENT_VAR_12345"` → `""`
- `env_or "NONEXISTENT" "default"` → `"default"`
- `synoema run file.sno -- a b c` → `args == ["a" "b" "c"]`
- All 864 existing tests pass, 0 warnings
- BPE: `env`, `env_or`, `args` are existing keywords — no new tokens needed

## What is NOT in scope

- JIT support (interpreter-first)
- `setenv` / mutation
- Process exit codes
- Stdin piping / redirection flags
