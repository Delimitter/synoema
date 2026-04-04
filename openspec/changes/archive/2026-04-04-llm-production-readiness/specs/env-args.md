# Spec: Environment Variables + CLI Arguments

## Environment Variables

```sno
env : String -> String      -- пустая строка если не найдена
env_or : String -> String -> String  -- env key default
```

Interpreter-only. JIT — вне scope (env доступен при старте, не в hot loop).

### Реализация
FFI в eval.rs: `std::env::var(key).unwrap_or_default()`

## CLI Arguments

```sno
args : [String]             -- список аргументов (без имени программы)
```

Аргументы после `--` в CLI:
```bash
synoema run server.sno -- --port 8080 --host 0.0.0.0
```

### Реализация
В eval.rs: inject `args` в top-level env перед eval программы. Значение берётся из `std::env::args()`, фильтруя всё до `--`.

## Что НЕ входит

- Set env (env mutation)
- Typed CLI parsing (ala clap/argparse)
- Environment file loading (.env)
