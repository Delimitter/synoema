# Design: Built-in Testing

## Pipeline changes

```
Source → Lexer → Parser → TypeChecker → Eval
             ↓        ↓         ↓          ↓
         KwTest    Decl::Test  infer body  run_tests()
         KwProp    ExprKind::   as Bool
         KwImplies  Prop/Implies
```

## 1. Lexer (synoema-lexer)

Три новых keyword-токена:
- `Token::KwTest` — keyword `test`
- `Token::KwProp` — keyword `prop`
- `Token::KwImplies` — keyword `implies`

Scanner: добавить в keyword match (`"test" => Token::KwTest`, etc.).

BPE: все три — 1 токен в cl100k_base (верифицировано).

## 2. Parser (synoema-parser)

### AST additions

```rust
enum Decl {
    // ... existing ...
    Test {
        name: String,
        body: Expr,
        span: Span,
    },
}

enum ExprKind {
    // ... existing ...
    /// prop x y -> body (property generator)
    Prop(Vec<String>, Box<Expr>),
    /// cond implies body
    Implies(Box<Expr>, Box<Expr>),
}
```

### Parsing

**Test declaration** — парсится в `parse_program_recovering` при встрече `KwTest`:
```
test <Str> = <expr>
```

**Prop expression** — парсится как prefix в Pratt parser (binding power ниже `==`):
```
prop <lowerId>+ -> <expr>
```

**Implies** — парсится как infix operator с приоритетом НИЖЕ `==` но ВЫШЕ `&&`/`||`:
```
<expr> implies <expr>
```

Приоритет implies: 1 (ниже `||` = 2). Это обеспечивает `a == b implies c == d` парсится как `(a == b) implies (c == d)`.

## 3. Type checker (synoema-types)

- `Decl::Test`: infer body, unify result с `Bool`. Ошибка если тело не Bool.
- `ExprKind::Prop`: каждая переменная получает fresh type variable. Тело: Bool. Результат: Bool.
- `ExprKind::Implies`: оба операнда Bool, результат Bool.

## 4. Desugar (synoema-core)

- `Decl::Test` → сохраняется как метаданные, не десахаризуется в Core IR
- `Prop` → не десахаризуется (не используется в JIT)
- `Implies` → не десахаризуется (не используется в JIT)

## 5. Eval (synoema-eval)

### Test execution

`eval_program` возвращает test declarations вместе с результатами. Новая функция:
```rust
pub fn extract_tests(program: &Program) -> Vec<TestDecl>
```

Runner в REPL вызывает eval для каждого теста в контексте всех деклараций.

### Prop evaluation

`eval_prop(vars, body, env)`:
1. Определить тип каждой переменной из type inference
2. Сгенерировать 100 наборов значений (thread_rng)
3. Для каждого набора: подставить в env → eval body → check == Bool(true)
4. Если fail → вернуть counterexample

### Implies evaluation

`eval_implies(cond, body)`:
- cond == false → Discard (не fail, не pass)
- cond == true → eval body → result

### Type-to-generator mapping

```rust
fn generate_value(ty: &Type, rng: &mut impl Rng) -> Value {
    match ty {
        Type::Con("Int") => Value::Int(rng.gen_range(-100..=100)),
        Type::Con("Bool") => Value::Bool(rng.gen_bool(0.5)),
        Type::Con("String") => random_ascii_string(0..=8, rng),
        Type::App(List, elem) => random_list(elem, 0..=10, rng),
        _ => panic!("Cannot generate values for type {ty}"),
    }
}
```

**Зависимость на rand**: НЕ добавляем. Используем простой LCG (linear congruential generator) в eval, ~10 строк. Это соответствует правилу минимализма зависимостей.

## 6. REPL runner (synoema-repl)

Расширить `run_doctests`:
1. После extract_doctests — также extract test declarations из AST
2. Если `--filter <str>` — фильтровать по name.contains(str)
3. Для каждого `Decl::Test`: eval body в контексте программы
4. Для prop-тестов: eval с генерацией, показать counterexample при fail
5. Вывод: `test "name" ... ok/FAILED`

## 7. GBNF grammar

Добавить:
```
decl ::= func-def | type-sig | type-def | type-alias | test-decl
test-decl ::= "test" ws string-lit ws "=" ws expr
```

## 8. Documentation

| Что обновить | Содержимое |
|-------------|-----------|
| docs/llm/synoema.md | Секция testing: test/prop/implies синтаксис |
| docs/specs/language_reference.md | §test declarations, §property testing |
| docs/testing.md | Описание встроенных тестов |
| CLAUDE.md | Обновить счётчик тестов |
| context/PROJECT_STATE.md | Добавить built-in testing |
| context/PHASES.md | Добавить фазу |

## Decisions

1. **Нет rand crate** — собственный LCG для генерации (правило минимализма)
2. **implies вместо ==>** — 1 BPE-токен vs 3
3. **prop вместо forall** — короче, 1 токен, менее перегружен математической семантикой
4. **100 итераций** — захардкожено, достаточно для обнаружения типичных ошибок
5. **Только interpreter** — JIT не нужен для тестов
6. **Тесты не экспортируются** — `test` живёт только в файле, невидим снаружи
