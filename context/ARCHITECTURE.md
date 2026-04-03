# Архитектура Synoema

## Pipeline

```
Source (.sno) → Lexer → Parser → Types (HM) → Core IR → Optimizer → Eval (interpreter)
                                                                   → Codegen (Cranelift JIT)
```

## Crates

| Crate | Назначение | LOC |
|-------|-----------|-----|
| synoema-lexer | Токенизация, offside rule | ~735 |
| synoema-parser | Pratt parser, 15 ExprKind | ~1672 |
| synoema-types | Hindley-Milner (Algorithm W) | ~1908 |
| synoema-core | Core IR (System F), optimizer | ~1536 |
| synoema-eval | Tree-walking interpreter | ~1894 |
| synoema-codegen | Cranelift JIT + runtime | ~3044 |
| synoema-repl | CLI: run/jit/eval/REPL | ~271 |

## Ключевые файлы по задачам

| Задача | Файл(ы) |
|--------|---------|
| Новый оператор/синтаксис | `lexer/src/lexer.rs` → `parser/src/parser.rs` |
| Новый тип данных | `types/src/types.rs` + `types/src/infer.rs` |
| Новая десахаризация | `core/src/desugar.rs` |
| Новый PrimOp в JIT | `codegen/src/compiler.rs` (`compile_binop`/`compile_unop`) |
| Новый runtime FFI | `codegen/src/runtime.rs` + регистрация в `compiler.rs` |
| Новый pattern в JIT | `codegen/src/compiler.rs` (`compile_case`) |
| Новая CLI команда | `repl/src/main.rs` |
| GBNF-грамматика | `tools/constrained/synoema.gbnf` |
| BPE-верификация | `tools/bpe-verify/verify_bpe.py` |

## Tagged Pointer ABI (JIT)

Все значения — `i64`. Тип определяется по тегам:
- `bit 0 = 1` → List (Cons/Nil node)
- `bit 1 = 1` → String (StrNode)
- `CON_TAG (0x01)` → ADT constructor (ConNode)
- `FLOAT_TAG (0x04)` → Float (FloatNode)
- `RECORD_TAG (0x05)` → Record (RecordNode)
- Иначе → Int / Bool (unboxed)

## Runtime FFI паттерн

```rust
// 1. runtime.rs: добавить extern "C" fn
pub extern "C" fn synoema_my_func(arg: i64) -> i64 { ... }

// 2. compiler.rs → Compiler::new(): зарегистрировать символ
builder.symbol("synoema_my_func", runtime::synoema_my_func as *const u8);

// 3. compiler.rs → declare_runtime_functions(): объявить сигнатуру
decl(self, "synoema_my_func", "my_func", &sig1)?;
```
