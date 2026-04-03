# Proposal: String Interpolation

## Problem Statement

Synoema не поддерживает строковую интерполяцию. Сейчас конкатенация строк с выражениями требует ручного вызова `show` и оператора `++`:

```sno
msg = "Hello " ++ name ++ ", you have " ++ show count ++ " items"
```

Это 11 токенов. С интерполяцией:

```sno
msg = "Hello ${name}, you have ${count} items"
```

Это 1 токен (одна строка). Экономия ~10x для строковых конструкций.

## Syntax Choice

Delimiter: `${expr}` — JavaScript/Kotlin-стиль:
- `${` = 1 BPE-токен (cl100k_base: 2420). ✓ BPE-aligned
- `}` = 1 BPE-токен (cl100k_base: 92). ✓ BPE-aligned
- Максимальная знакомость для LLM (обучены на JS/Kotlin/Dart/Shell)
- Escape: `\$` для литерального `$`

## Desugaring

Интерполяция — чистый синтаксический сахар. Десахаризация на уровне **парсера**:

```
"text ${expr} more" → "text " ++ show expr ++ " more"
```

- `show` для String возвращает строку as-is (уже работает в interpreter и JIT)
- Никаких изменений в type checker, Core IR, interpreter, JIT
- Вся работа в lexer (токенизация) + parser (десахаризация)

## Scope

- Lexer: расширить `scan_string` для обнаружения `${`
- Parser: десахарить интерполированные строки в `show` + `++`
- Нет новых ExprKind, нет новых CoreExpr, нет изменений в runtime
- 2 файла для кода, 4-5 файлов для тестов/документации

## Success Criteria

- `"x=${x}"` десахарится в `"x=" ++ show x`
- Вложенные `${}` работают: `"a ${b ++ "c ${d}"} e"`
- `\$` escape работает: `"price is \$5"` → `"price is $5"`
- Все 719+ тестов зелёные, 0 warnings
- BPE-verified: `${` = 1 токен
