# Инструкции для LLM-разработчика

> Этот документ объясняет, как продолжать разработку Synoema.
> Прочитай PROJECT_STATE.md перед этим файлом.

---

## Принципы разработки

1. **Тесты перед кодом.** Каждая новая фича — сначала тест, потом реализация. Текущий стандарт: 488 тестов, 0 ошибок.

2. **Нет warnings.** `cargo build` должен быть чистым. Единственное исключение — `unused fresh` в desugar.rs (документирован в PROJECT_STATE.md).

3. **Минимальные зависимости.** Не добавлять crates без крайней необходимости. Текущие зависимости: только Cranelift + pretty_assertions.

4. **BPE-aligned.** Любой новый оператор или ключевое слово ОБЯЗАН быть одним BPE-токеном на cl100k_base. Проверить: `tools/bpe-verify/verify_bpe.py`.

5. **Два backend'а.** Interpreter — reference implementation (все фичи). JIT — high-performance (поднабор фич). Новая фича сначала в interpreter, потом в JIT.

6. **Коммит = тесты проходят.** Никогда не коммитить с красными тестами.

---

## Как добавить новую фичу: пошаговый шаблон

### Пример: добавить оператор `**` (возведение в степень)

**Шаг 1: Lexer** (`crates/synoema-lexer/src/lexer.rs`)
```rust
// Добавить токен
Token::StarStar => "**",
// Добавить в scan_operator()
```

**Шаг 2: Parser** (`crates/synoema-parser/src/parser.rs`)
```rust
// Добавить в таблицу приоритетов
Token::StarStar => (Prec::Exp, Assoc::Right),
// Добавить в parse_infix()
BinOp::Pow => ...
```

**Шаг 3: Types** (`crates/synoema-types/src/infer.rs`)
```rust
// Добавить типовое правило
BinOp::Pow => (Type::Int, Type::Int, Type::Int),
```

**Шаг 4: Core IR** (`crates/synoema-core/src/core_ir.rs`)
```rust
// Добавить PrimOp
PrimOp::Pow => "pow#",
```

**Шаг 5: Interpreter** (`crates/synoema-eval/src/eval.rs`)
```rust
// Добавить eval
PrimOp::Pow => Value::Int(l.pow(r as u32)),
```

**Шаг 6: JIT** (`crates/synoema-codegen/src/compiler.rs`)
```rust
// Добавить в compile_binop (или как runtime FFI)
```

**Шаг 7: Тесты.** Добавить тест в каждый crate, который затронут.

**Шаг 8: GBNF.** Обновить `tools/constrained/synoema.gbnf`.

**Шаг 9: BPE-verify.** Проверить, что `**` = 1 BPE-токен.

---

## Приоритеты развития (ROADMAP)

### КРИТИЧЕСКИЕ (разблокируют другие фичи)

#### Phase 9.2: Closures в JIT
**Что:** indirect function calls, function pointers как значения
**Зачем:** разблокирует map, filter, [x | x <- xs] через JIT
**Файлы:** `codegen/src/compiler.rs`, `codegen/src/runtime.rs`
**Подход:**
- Closure = struct { function_ptr, env_ptr }
- Env = heap-allocated массив захваченных переменных
- Call = load function_ptr from closure struct, call with (env_ptr, args...)
- Runtime FFI: `synoema_make_closure(fn_ptr, env_ptr) -> closure_ptr`
**Тесты для написания:**
```
map (\x -> x * 2) [1 2 3]                → [2 4 6]
filter (\x -> x > 3) [1 2 3 4 5]         → [4 5]
[x * 2 | x <- [1 2 3]]                   → [2 4 6]
(\f -> \x -> f (f x)) (\x -> x + 1) 0    → 2
```
**Сложность:** Высокая. ~2 недели.

#### Phase 10.1: Tail Call Optimization (TCO)
**Что:** tail calls не создают новый stack frame
**Зачем:** euler1 (рекурсия 999) и подобные не падают
**Файлы:** `codegen/src/compiler.rs`
**Подход:**
- Обнаружить tail position в Core IR
- В Cranelift: заменить `call` + `return` на `jump` к началу функции с новыми аргументами
- Для interpreter: trampolining или loop detection
**Тесты:**
```
go 0 = 0; go n = go (n - 1)
go 1000000 → 0  (не stack overflow)
```
**Сложность:** Средняя. ~1 неделя.

### ВАЖНЫЕ (расширяют возможности языка)

#### Phase 9.3: Строки в JIT
**Что:** String как heap-allocated UTF-8 буфер
**Зачем:** fizzbuzz через JIT, show для строк
**Файлы:** `codegen/src/runtime.rs`, `codegen/src/compiler.rs`
**Подход:**
- Строка = struct { ptr: *u8, len: usize }
- Runtime: `synoema_str_new(ptr, len)`, `synoema_str_concat`, `synoema_str_print`
- Литералы: встроить в data section, передать ptr+len
**Сложность:** Средняя. ~1 неделя.

#### Phase 9.4: Records + Row Polymorphism
**Что:** `{name = "Alice", age = 30}`, field access с типовыми гарантиями
**Файлы:** ВСЕ crates (новый тип данных)
**Подход:**
- Lexer: `{`, `}`, field access `.`
- Parser: RecordExpr, FieldAccess
- Types: Row types (Daan Leijen, "Extensible Records with Scoped Labels")
- Core IR: Record → struct
- JIT: heap-allocated struct с offset-based access
**Сложность:** Средняя. ~2 недели.

#### Phase 9.5: Модули
**Что:** `mod Math`, `use Math (sqrt pi)`, раздельная компиляция
**Файлы:** новый crate `synoema-modules` или расширение parser + eval
**Подход:**
- `mod Name` → namespace
- `use Name (item1 item2)` → import
- Файловая система: `math.sno` → `mod Math`
- Раздельная компиляция: каждый модуль → отдельный Core IR
**Сложность:** Средняя. ~2 недели.

#### Phase 9.6: IO / Effects
**Что:** `@io` маркер, чтение файлов, stdin/stdout
**Подход:**
- IO monad (как Haskell) или effect markers
- `readFile : String -> @io String`
- `print : a -> @io ()`
- `main : @io Int`
**Сложность:** Средняя. ~1 неделя.

#### Phase 9.7: FFI
**Что:** `@native "printf"` — вызов C-функций
**Подход:**
- Декларация: `@native "strlen" : String -> Int`
- Cranelift: declare_function с Linkage::Import
- dlopen/dlsym для динамической загрузки
**Сложность:** Низкая. ~1 неделя.

### ОПТИМИЗАЦИИ

#### Phase 10.2: Constant Folding / DCE
**Что:** `2 + 3` → `5` на этапе компиляции, удаление мёртвого кода
**Файлы:** новый pass в `synoema-core/src/optimize.rs`
**Сложность:** Средняя. ~1 неделя.

#### Phase 10.3: Region-Based Memory
**Что:** автоматическое управление памятью без GC (как MLkit)
**Зачем:** сейчас JIT runtime leaks memory (malloc без free)
**Подход:** region inference на Core IR, каждый region = arena allocator
**Сложность:** Высокая. ~3 недели.

#### Phase 10.4: LLVM Backend
**Что:** `--backend llvm` для максимальной производительности
**Подход:** inkwell (Rust bindings to LLVM), альтернативный codegen
**Сложность:** Высокая. ~4 недели.

### ИНСТРУМЕНТЫ

#### VS Code Extension
**Что:** подсветка синтаксиса для .sno файлов
**Подход:** TextMate grammar (.tmLanguage.json)
**Сложность:** Низкая. ~1 день.

#### LSP Server
**Что:** автокомплит, go-to-definition, inline errors
**Подход:** tower-lsp crate, интеграция с synoema-types
**Сложность:** Высокая. ~3 недели.

#### Web Playground
**Что:** попробовать Synoema в браузере
**Подход:** WASM-компиляция interpreter'а (wasm-bindgen)
**Сложность:** Средняя. ~2 недели.

## Паттерны кодирования

### Добавить новый тест в JIT:
```rust
// В crates/synoema-codegen/src/lib.rs
#[test]
fn my_new_test() {
    assert_eq!(jit("main = ...your code..."), expected_i64);
}
```

### Добавить новый runtime FFI:
```rust
// 1. В runtime.rs: добавить extern "C" fn
pub extern "C" fn synoema_my_func(arg: i64) -> i64 { ... }

// 2. В compiler.rs → Compiler::new(): зарегистрировать символ
builder.symbol("synoema_my_func", runtime::synoema_my_func as *const u8);

// 3. В compiler.rs → declare_runtime_functions(): объявить сигнатуру
decl(self, "synoema_my_func", "my_func", &sig1)?;  // sig1 = fn(i64)->i64
```

### Debug Core IR:
```rust
// Добавить в тест:
let prog = synoema_parser::parse(src).unwrap();
let core = synoema_core::desugar_program(&prog);
for d in &core.defs {
    println!("{}: {}", d.name, d.body);
}
```

---

## Контакты и ресурсы

- **Автор:** Андрей (IT-executive, Москва, стек Rust/Vue/PostgreSQL)
- **Язык кодовой базы:** Rust
- **Стиль кода:** идиоматический Rust, без unsafe (кроме FFI в runtime.rs)
- **Формат коммитов:** `phase X.Y: краткое описание`
