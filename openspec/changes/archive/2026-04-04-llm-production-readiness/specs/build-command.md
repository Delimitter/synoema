# Spec: Build Command (`synoema build`)

## CLI

```bash
synoema build                      # use project.sno entry point
synoema build --entry src/main.sno # explicit entry point
```

## Поведение

1. Если `project.sno` существует → читает `entry` binding → использует как entry point
2. Resolve all `import` statements transitively
3. Type check all files together (shared type env)
4. Report all errors across all files at once (error recovery)
5. Output: ничего (type check only), exit 0 on success

### С JIT

```bash
synoema build --jit                # type check + JIT compile + run
```

## project.sno Parsing

project.sno — обычный .sno файл. `synoema build` ищет binding `entry`:
```sno
name = "myapp"
version = "0.1.0"
entry = "src/main.sno"
```

Если `entry` не найден → error "no entry point".

## Multi-file Resolution

Использует существующий `import "path.sno"` механизм:
1. Начинает с entry file
2. Рекурсивно resolve imports (cycle detection, diamond caching — уже есть)
3. Собирает все diagnostics со всех файлов

## Что НЕ входит

- Compilation to binary (ahead-of-time)
- Incremental compilation
- Dependency fetching (no package registry)
- Output directory / artifacts
