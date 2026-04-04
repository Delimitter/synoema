# Spec: JSON Parsing

## Текущее состояние

`json_escape : String -> String` уже существует (output only). Нет парсинга JSON → Synoema values.

## Design Decisions

### D1. Interpreter-only, не JIT

JSON parsing — I/O boundary операция, не hot loop. Реализуется как builtin в eval.rs. Конструирует `Value::Con` дерево напрямую из Rust. JIT не затрагивается.

### D2. Ручной парсер, не serde_json

Правило минимализма: никаких новых зависимостей. Recursive descent JSON parser ~200 строк в eval.rs. Достаточен для well-formed JSON (RFC 8259).

### D3. JsonValue через prelude ADT, а не hardcoded

```sno
JsonValue = JNull
          | JBool Bool
          | JNum Float
          | JStr String
          | JArr [JsonValue]
          | JObj (Map String JsonValue)
```

Определяется в `prelude.sno`. Parser в eval.rs конструирует `Value::Con("JStr", [Value::Str(s)])` и т.д.

## ADT (в prelude)

```sno
JsonValue = JNull
          | JBool Bool
          | JNum Float
          | JStr String
          | JArr [JsonValue]
          | JObj (Map String JsonValue)
```

## API

### Core (builtins в eval.rs)
```sno
json_parse  : String -> Result JsonValue String
json_encode : JsonValue -> String
```

### Accessors (в prelude, pure functions)
```sno
json_get : String -> JsonValue -> Result JsonValue String
json_get k (JObj m) = map_lookup k m
json_get _ _        = Err "not an object"

json_arr : JsonValue -> Result [JsonValue] String
json_arr (JArr xs) = Ok xs
json_arr _         = Err "not an array"

json_str : JsonValue -> Result String String
json_str (JStr s) = Ok s
json_str _        = Err "not a string"

json_num : JsonValue -> Result Float String
json_num (JNum n) = Ok n
json_num _        = Err "not a number"

json_bool : JsonValue -> Result Bool String
json_bool (JBool b) = Ok b
json_bool _         = Err "not a boolean"

json_is_null : JsonValue -> Bool
json_is_null JNull = true
json_is_null _     = false
```

## Реализация: json_parse в eval.rs

Builtin `json_parse` в `call_builtin`:

```rust
"json_parse" => {
    let input = sval(&args[0])?;
    match json::parse_json(&input) {
        Ok(val) => Ok(Value::Con("Ok".into(), vec![val])),
        Err(msg) => Ok(Value::Con("Err".into(), vec![Value::Str(msg)])),
    }
}
```

### Value construction mapping

| JSON | Synoema Value |
|------|---------------|
| `null` | `Value::Con("JNull", vec![])` |
| `true` | `Value::Con("JBool", vec![Value::Bool(true)])` |
| `3.14` | `Value::Con("JNum", vec![Value::Float(3.14)])` |
| `"hello"` | `Value::Con("JStr", vec![Value::Str("hello")])` |
| `[1, 2]` | `Value::Con("JArr", vec![Value::List([JNum(1.0), JNum(2.0)])])` |
| `{"k": "v"}` | `Value::Con("JObj", vec![map_value])` |

### Map construction for JObj

JObj содержит `Map String JsonValue`. В Rust это:
```rust
// MkMap [MkPair "key" json_val, ...]
fn make_map(pairs: Vec<(String, Value)>) -> Value {
    // Sort by key, wrap in MkPair, wrap in MkMap
    let mut sorted = pairs;
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let pair_values: Vec<Value> = sorted.into_iter()
        .map(|(k, v)| Value::Con("MkPair".into(), vec![Value::Str(k), v]))
        .collect();
    Value::Con("MkMap".into(), vec![Value::List(pair_values)])
}
```

### JSON parser structure (~200 LOC)

```rust
mod json {
    use super::Value;

    pub fn parse_json(input: &str) -> Result<Value, String> {
        let mut chars = input.trim().chars().peekable();
        let val = parse_value(&mut chars)?;
        // skip trailing whitespace
        skip_ws(&mut chars);
        if chars.peek().is_some() {
            return Err("trailing characters after JSON value".into());
        }
        Ok(val)
    }

    fn parse_value(chars: &mut Peekable<Chars>) -> Result<Value, String> {
        skip_ws(chars);
        match chars.peek() {
            Some('"') => parse_string(chars),
            Some('{') => parse_object(chars),
            Some('[') => parse_array(chars),
            Some('t') | Some('f') => parse_bool(chars),
            Some('n') => parse_null(chars),
            Some(c) if c.is_ascii_digit() || *c == '-' => parse_number(chars),
            Some(c) => Err(format!("unexpected character: {}", c)),
            None => Err("unexpected end of input".into()),
        }
    }
    // ... parse_string, parse_object, parse_array, parse_number, parse_bool, parse_null
}
```

Поддержка escape sequences: `\"`, `\\`, `\/`, `\n`, `\t`, `\r`, `\b`, `\f`, `\uXXXX`.

## Реализация: json_encode в eval.rs

Builtin `json_encode`:
```rust
"json_encode" => {
    let encoded = json::encode_json(&args[0])?;
    Ok(Value::Str(encoded))
}
```

Рекурсивный walk по Value tree:
- `JNull` → `"null"`
- `JBool true` → `"true"`
- `JNum 3.14` → `"3.14"`
- `JStr "hello"` → `"\"hello\""` (с escaping)
- `JArr [...]` → `"[...]"`
- `JObj map` → `"{...}"`

## Пример использования

```sno
main =
  data = json_parse "{\"name\": \"Alice\", \"age\": 30}"
  name = data |> and_then (json_get "name") |> and_then json_str |> unwrap_or "unknown"
  age = data |> and_then (json_get "age") |> and_then json_num |> unwrap_or 0.0
  print ("Name: " ++ name ++ ", Age: " ++ show age)
```

## Зависимости

- `Result a e` (prelude) — для return type
- `Map String JsonValue` (prelude) — для JObj
- `Pair` (prelude) — внутри Map

## Type checker changes

Добавить в `builtin_env()` в infer.rs:
```rust
// json_parse: String -> Result JsonValue String
// json_encode: JsonValue -> String
```

Типы `JsonValue`, `Map`, `Pair` приходят из prelude (как ADTs). Builtin types нужно выразить через `Type::Con("Result")`, `Type::App(...)` etc.

**Проблема:** type checker's `builtin_env()` вызывается ДО парсинга prelude (prelude = часть source). Значит `JsonValue` ещё не зарегистрирован как тип.

**Решение:** НЕ добавлять `json_parse` / `json_encode` в type checker's `builtin_env()`. Вместо этого — определить их как обычные функции-обёртки в prelude:

```sno
--- Parse JSON string. Returns Result JsonValue String.
--- (implemented as builtin, wrapper for type signature)
json_parse_raw : String -> String  -- raw builtin, returns serialized
-- Нет, это не работает...
```

**Альтернативное решение:** добавить `json_parse` и `json_encode` в type checker ПОСЛЕ регистрации ADTs. Текущий flow в infer.rs:
1. `builtin_env()` → base types
2. `infer_program()` → first pass: `register_adt()` для всех ADTs (включая prelude)
3. second pass: infer functions

Нужно добавить шаг 1.5: после register_adt, вставить типы `json_parse` и `json_encode` в env. Или: просто определить json_parse в builtin_env с generic type и let type checker unify.

**Simplest solution:** в builtin_env() typed как `∀a. String → a` (как error). Пользователь получит правильный тип через unification при использовании. Или ещё проще — typed как `String → String` в builtin_env (wrong but won't crash), а реальный тип приходит из prelude wrapper:

```sno
-- в prelude:
json_parse s = json_parse_raw s   -- где json_parse_raw : String -> Result JsonValue String builtin
```

**Final decision:** добавить `json_parse_raw` и `json_encode_raw` как builtins с упрощёнными типами. В prelude — обёртки `json_parse` и `json_encode` с правильными типами. Type safety обеспечивается prelude-уровнем.

## Registering builtins

В eval.rs `builtin_env()`:
```rust
("json_parse_raw", 1),   // String -> <any> (actually Result JsonValue String)
("json_encode_raw", 1),  // <any> -> String (actually JsonValue -> String)
```

В infer.rs `builtin_env()`:
```rust
// json_parse_raw: ∀a. String -> a (like error — polymorphic return)
// json_encode_raw: ∀a. a -> String (like show — polymorphic input)
```

В prelude:
```sno
json_parse : String -> Result JsonValue String
json_parse s = json_parse_raw s

json_encode : JsonValue -> String
json_encode v = json_encode_raw v
```

Prelude обёртки дают правильные типы через let-generalization + unification.

## Что НЕ входит

- JIT support (interpreter-only)
- JSON Schema validation
- JSON Path queries
- Streaming parsing
- Pretty-print (compact only)
- Number precision (uses f64)
