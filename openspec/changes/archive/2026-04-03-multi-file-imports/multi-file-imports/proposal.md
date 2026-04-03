---
id: proposal
type: proposal
status: draft
---

# Proposal: Multi-File Imports

## Problem Statement

Synoema модули (`mod Math ... end`) существуют только inline в одном файле. Невозможно разбить код на файлы — это блокирует любой проект больше одного файла.

```sno
-- Сейчас: всё в одном файле
mod Math
  square x = x * x
end

main = Math.square 5
```

```sno
-- Нужно: разделение по файлам
-- math.sno:
mod Math
  square x = x * x
end

-- main.sno:
import "math.sno"
use Math (square)

main = square 5
```

## Prior Art

Архивная спецификация: `openspec/changes/archive/2026-04-03-multi-file-imports/specs/multi-file-imports.md`

## Scope

- `import "path.sno"` — загрузка деклараций из другого файла
- Рекурсивное разрешение с cycle detection и diamond caching
- Работает в interpreter и JIT (merged program approach)
- ~200 LOC нового кода, ≥8 тестов
- 6 файлов модифицированы

## BPE Impact

```
import = 1 BPE token (cl100k_base) ✅
```

## Success Criteria

- [ ] `import "file.sno"` загружает и мержит декларации
- [ ] Imported `mod` блоки доступны через `use`
- [ ] Circular imports → diagnostic error
- [ ] Diamond imports → файл загружен один раз (cached by absolute path)
- [ ] Relative paths resolved from importing file's directory
- [ ] Работает в interpreter и JIT
- [ ] GBNF grammar обновлена
- [ ] `docs/llm/synoema.md` обновлена
- [ ] Example files созданы
- [ ] ≥8 тестов, all green, 0 warnings
