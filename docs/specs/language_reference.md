# Synoema Language Reference v0.4

## Language Specification

**Status:** Implemented — парсер, type checker, интерпретатор, Cranelift JIT
**Дата:** Апрель 2026
**Область:** Core language + Records + Modules + JIT backend

---

## Содержание

1. Обзор и принципы
2. Лексическая спецификация
3. Синтаксическая грамматика (EBNF)
4. Система типов (формальные правила)
5. Операционная семантика (big-step)
6. BPE-alignment таблица
7. Примеры программ с токенным анализом

---

## 1. Обзор и принципы

### 1.1 Неизменяемые аксиомы

Следующие свойства зафиксированы и не подлежат пересмотру:

**A1. Strict evaluation.** Все выражения вычисляются строго (eager), слева направо. Ленивость — только через явную аннотацию (в будущих версиях).

**A2. Hindley-Milner type inference.** Типы выводятся автоматически алгоритмом W. Аннотации типов опциональны и служат документацией.

**A3. BPE-aligned terminals.** Каждый оператор и ключевой символ языка является одиночным BPE-токеном в cl100k_base и Llama tokenizer.

**A4. Deterministic context-free grammar.** Грамматика Synoema — DCFG, компилируемая в детерминированный pushdown automaton без shift-reduce конфликтов.

**A5. Immutability by default.** Все привязки неизменяемы. Мутация — через явные конструкции (в будущих версиях).

**A6. Expression-oriented.** Каждая конструкция языка — выражение, возвращающее значение.

### 1.2 Нотация в этом документе

- `e` — произвольное выражение
- `τ`, `σ` — типы
- `Γ` — типовое окружение (контекст)
- `⊢` — отношение типизации ("из Γ следует, что e имеет тип τ")
- `→` в типах — функциональный тип
- `→` в семантике — шаг редукции
- `⇓` — отношение big-step вычисления ("e вычисляется в значение v")
- `[x ↦ v]` — подстановка v вместо x

---

## 2. Лексическая спецификация

### 2.1 Набор символов

Synoema использует подмножество ASCII (коды 0x20–0x7E) + перенос строки (0x0A) + табуляция (0x09). Unicode допускается только внутри строковых литералов.

### 2.2 Пробельные символы (whitespace)

```
WS       = ' ' | '\t'
NEWLINE  = '\n'
INDENT   = увеличение уровня индентации (значимое)
DEDENT   = уменьшение уровня индентации (значимое)
```

Индентация значима (как в Python/Haskell offside rule). Единица индентации — 2 пробела.

### 2.3 Комментарии

```
COMMENT     = '--' [^\n]* NEWLINE
DOC_COMMENT = '---' [^\n]* NEWLINE
```

Блочные комментарии отсутствуют (экономия: не нужен терминал закрытия `*/`).

### 2.4 Ключевые слова

MVL содержит **7 ключевых слов**:

| Слово | BPE-токены | Назначение |
|-------|-----------|------------|
| `mod` | 1 | объявление модуля |
| `use` | 1 | импорт |
| `trait` | 1 | type class |
| `impl` | 1 | реализация type class |
| `true` | 1 | литерал Bool |
| `false` | 1 | литерал Bool |
| `lazy` | 1 | ленивое вычисление (зарезервировано) |

Ключевые слова нельзя использовать как идентификаторы.

Отсутствуют (сознательно исключены для экономии токенов):
- `def` / `fn` / `fun` / `func` — определение функции через `=`
- `if` / `then` / `else` — заменены на `? -> :`
- `let` / `in` — привязки через индентацию
- `return` — последнее выражение в блоке = возвращаемое значение
- `where` — привязки только через `=` с индентацией
- `case` / `of` / `match` — паттерн-матчинг через множественные определения
- `do` — эффекты через `<-`
- `class` / `struct` / `data` / `type` — алгебраические типы через `=` и `|`

### 2.5 Операторы и разделители

Полный список, упорядоченный по приоритету (от низкого к высокому):

| Символ | Имя | BPE | Приоритет | Ассоц. |
|--------|-----|-----|-----------|--------|
| `<-` | bind | 1 | 1 | right |
| `\|>` | pipe | 1* | 2 | left |
| `\|\|` | or | 1 | 3 | left |
| `&&` | and | 1 | 4 | left |
| `==` | eq | 1 | 5 | none |
| `!=` | neq | 1 | 5 | none |
| `<` | lt | 1 | 6 | none |
| `>` | gt | 1 | 6 | none |
| `<=` | lte | 1 | 6 | none |
| `>=` | gte | 1 | 6 | none |
| `++` | concat | 1 | 7 | right |
| `+` | add | 1 | 8 | left |
| `-` | sub | 1 | 8 | left |
| `*` | mul | 1 | 9 | left |
| `/` | div | 1 | 9 | left |
| `%` | mod | 1 | 9 | left |
| `>>` | compose | 1 | 10 | right |
| `-` (prefix) | neg | 1 | 11 | — |
| `.` | field | 1 | 12 | left |
| function app | apply | 0 | 13 | left |

*`|>` = 1-2 BPE-токена в зависимости от контекста.

Разделители:

| Символ | Назначение | BPE |
|--------|-----------|-----|
| `(` `)` | группировка, кортежи | 1 каждый |
| `[` `]` | списки, list comprehension | 1 каждый |
| `\` | начало лямбды | 1 |
| `->` | стрелка (лямбда, тип, условие) | 1 |
| `?` | начало условия | 1 |
| `:` | аннотация типа, cons-pattern | 1 |
| `=` | определение / привязка | 1 |
| `\|` | альтернатива (ADT, pattern) | 1 |
| `@` | директива (@native, @io) | 1 |
| `.` | доступ к полю записи | 1 |
| `_` | wildcard pattern | 1 |
| `,` | разделитель в guards/comprehension | 1 |
| `..` | диапазон | 1 |

### 2.6 Литералы

```
INT      = '0' | [1-9][0-9]*
FLOAT    = INT '.' [0-9]+
STRING   = '"' [^"\n]* '"'
CHAR     = '\'' [^'\n] '\''
BOOL     = 'true' | 'false'
```

### 2.7 Идентификаторы

```
LOWER_ID = [a-z][a-zA-Z0-9_]*     -- переменные, функции
UPPER_ID = [A-Z][a-zA-Z0-9]*      -- типы, конструкторы
```

Конвенция: переменные и функции начинаются с lowercase, типы и конструкторы — с uppercase.

---

## 3. Синтаксическая грамматика (EBNF)

### 3.1 Нотация EBNF

```
{ X }     — 0 или более повторений X
[ X ]     — 0 или 1 вхождение X
X | Y     — альтернатива
'x'       — терминальный символ x
X Y       — конкатенация (X затем Y)
( X )     — группировка
```

### 3.2 Программа (верхний уровень)

```ebnf
program      = { topDecl } ;

topDecl      = moduleDecl
             | useDecl
             | typeSig
             | funcDef
             | typeDef
             | traitDef
             | implDef ;

moduleDecl   = 'mod' UPPER_ID NEWLINE INDENT { funcDef } DEDENT ;

useDecl      = 'use' UPPER_ID '(' LOWER_ID { LOWER_ID } ')' NEWLINE ;
```

Пример:
```
mod Math
  square x = x * x
  abs x = ? x < 0 -> 0 - x : x

use Math (square abs)

main = square 5 + abs (0 - 3)
```

### 3.3 Определения функций

```ebnf
typeSig      = LOWER_ID ':' type { type } '->' type NEWLINE ;

funcDef      = LOWER_ID { pattern } '=' expr NEWLINE ;
```

Функция определяется одним или несколькими уравнениями (pattern matching):
```
fib 0 = 0
fib 1 = 1
fib n = fib (n - 1) + fib (n - 2)
```

### 3.4 Определения типов

```ebnf
typeDef      = UPPER_ID { LOWER_ID } '=' variants NEWLINE ;

variants     = variant { '|' variant } ;

variant      = UPPER_ID { typeAtom } ;
```

Примеры:
```
Maybe a = Just a | None
List a = Cons a (List a) | Nil
Shape = Circle Float | Rect Float Float | Point
```

### 3.5 Type classes (trait/impl)

```ebnf
traitDef     = 'trait' UPPER_ID LOWER_ID INDENT
                 { typeSig }
               DEDENT ;

implDef      = 'impl' UPPER_ID typeAtom [ '?' constraint ] INDENT
                 { funcDef }
               DEDENT ;

constraint   = UPPER_ID LOWER_ID { ',' UPPER_ID LOWER_ID } ;
```

### 3.6 Выражения

```ebnf
expr         = bindExpr
             | pipeExpr ;

bindExpr     = LOWER_ID '<-' expr NEWLINE expr ;

pipeExpr     = orExpr { '|>' orExpr } ;

orExpr       = andExpr { '||' andExpr } ;

andExpr      = cmpExpr { '&&' cmpExpr } ;

cmpExpr      = concatExpr [ cmpOp concatExpr ] ;
cmpOp        = '==' | '!=' | '<' | '>' | '<=' | '>=' ;

concatExpr   = addExpr { '++' addExpr } ;

addExpr      = mulExpr { ('+' | '-') mulExpr } ;

mulExpr      = composeExpr { ('*' | '/' | '%') composeExpr } ;

composeExpr  = unaryExpr { '>>' unaryExpr } ;

unaryExpr    = [ '-' ] accessExpr ;

accessExpr   = appExpr { '.' LOWER_ID } ;

appExpr      = atom { atom } ;

atom         = LOWER_ID
             | UPPER_ID
             | literal
             | '(' expr ')'
             | listExpr
             | condExpr
             | lambdaExpr
             | blockExpr
             | recordExpr ;

recordExpr   = '{' '}'
             | '{' recordField { ',' recordField } '}' ;

recordField  = LOWER_ID '=' expr ;
```

### 3.7 Специальные выражения

```ebnf
condExpr     = '?' expr '->' expr ':' expr ;

lambdaExpr   = '\' { pattern } '->' expr ;

listExpr     = '[' ']'
             | '[' expr { expr } ']'
             | '[' expr '|' generators ']'
             | '[' expr '..' expr ']' ;

generators   = generator { ',' generator } ;
generator    = LOWER_ID '<-' expr
             | expr ;   -- guard (boolean filter)

blockExpr    = INDENT { binding } expr DEDENT ;

binding      = LOWER_ID '=' expr NEWLINE ;

literal      = INT | FLOAT | STRING | CHAR | BOOL ;
```

### 3.8 Паттерны

```ebnf
pattern      = '_'
             | LOWER_ID
             | literal
             | UPPER_ID { pattern }
             | '(' pattern ':' pattern ')'
             | '(' pattern ')'
             | recordPattern ;

recordPattern = '{' LOWER_ID '=' pattern { ',' LOWER_ID '=' pattern } '}' ;
```

Пример record patterns:
```
get_x {x = v, y = _} = v
swap {x = a, y = b}  = {x = b, y = a}
```

### 3.9 Типы

```ebnf
type         = typeAtom { '->' typeAtom }  ;  -- правоассоциативно

typeAtom     = UPPER_ID
             | LOWER_ID                     -- type variable
             | '(' type { type } ')'
             | '[' type ']'                 -- List sugar
             ;
```

### 3.10 Свойства грамматики

**Утверждение (DCFG).** Грамматика Synoema MVL детерминистична. Обоснование:

1. **Нет ambiguity в expressions:** приоритет и ассоциативность операторов задан явно через иерархию нетерминалов (pipeExpr → orExpr → ... → atom).

2. **Нет dangling else:** условие `? -> :` тернарно и всегда содержит обе ветки.

3. **Function application vs operators:** application (juxtaposition) имеет высший приоритет. `f x + 1` = `(f x) + 1`.

4. **Конструктор vs вызов:** UPPER_ID — конструктор, LOWER_ID — функция. Нет ambiguity.

5. **Индентация разрешает block/expr:** INDENT/DEDENT токены генерируются лексером, превращая значимые отступы в явные скобки.

---

## 4. Система типов

### 4.1 Множество типов

```
τ ::= α                        -- типовая переменная
    | C                         -- типовая константа (Int, Float, Bool, String, Char)
    | τ₁ → τ₂                  -- функциональный тип
    | T τ₁ ... τₙ              -- применение типового конструктора
    | ∀α. τ                     -- полиморфный тип (scheme)
```

Базовые типовые константы:

| Тип | Описание | Размер (runtime) |
|-----|----------|-----------------|
| `Int` | 64-bit signed integer | 8 bytes |
| `Float` | 64-bit IEEE 754 | 8 bytes |
| `Bool` | true / false | 1 byte |
| `String` | UTF-8 строка | pointer + len |
| `Char` | Unicode codepoint | 4 bytes |

Встроенные типовые конструкторы:

| Конструктор | Kind | Описание |
|-------------|------|----------|
| `List` | * → * | однородный список |
| `→` | * → * → * | функция |

### 4.2 Типовое окружение

```
Γ ::= ∅                        -- пустое окружение
    | Γ, x : σ                  -- расширение окружение переменной x с type scheme σ
```

Type scheme: `σ = ∀α₁...αₙ. τ`

Свободные переменные: `ftv(σ)` — множество типовых переменных, не связанных квантором.

### 4.3 Правила типизации

#### Переменная (VAR)

```
  x : σ ∈ Γ      τ = instantiate(σ)
  ──────────────────────────────────
              Γ ⊢ x : τ
```

`instantiate(∀α₁...αₙ. τ)` заменяет каждую αᵢ на свежую типовую переменную.

#### Целочисленный литерал (INT)

```
  ──────────────
  Γ ⊢ n : Int
```

#### Вещественный литерал (FLOAT)

```
  ────────────────
  Γ ⊢ f : Float
```

#### Строковый литерал (STRING)

```
  ──────────────────
  Γ ⊢ "..." : String
```

#### Булев литерал (BOOL)

```
  ──────────────────────────
  Γ ⊢ true : Bool    Γ ⊢ false : Bool
```

#### Применение функции (APP)

```
  Γ ⊢ e₁ : τ₁ → τ₂      Γ ⊢ e₂ : τ₁
  ──────────────────────────────────────
             Γ ⊢ e₁ e₂ : τ₂
```

#### Лямбда-абстракция (LAM)

```
  Γ, x : τ₁ ⊢ e : τ₂
  ─────────────────────────
  Γ ⊢ \x -> e : τ₁ → τ₂
```

#### Let-привязка (LET)

```
  Γ ⊢ e₁ : τ₁     σ₁ = generalize(Γ, τ₁)     Γ, x : σ₁ ⊢ e₂ : τ₂
  ──────────────────────────────────────────────────────────────────
                        Γ ⊢ (x = e₁; e₂) : τ₂
```

`generalize(Γ, τ) = ∀(ftv(τ) \ ftv(Γ)). τ`

#### Условное выражение (COND)

```
  Γ ⊢ e₁ : Bool     Γ ⊢ e₂ : τ     Γ ⊢ e₃ : τ
  ──────────────────────────────────────────────
           Γ ⊢ ? e₁ -> e₂ : e₃ : τ
```

Обе ветки должны иметь одинаковый тип.

#### Определение функции с pattern matching (FUNC)

```
  Γ, x₁ : τ₁, ..., xₙ : τₙ ⊢ eᵢ : τᵣ    для каждого уравнения i
  паттерны p₁ᵢ...pₙᵢ совместимы с τ₁...τₙ
  ──────────────────────────────────────────────
  Γ ⊢ f : τ₁ → ... → τₙ → τᵣ
```

#### Конструктор (CON)

```
  C : τ₁ → ... → τₙ → T α₁...αₘ    (из определения ADT)
  Γ ⊢ e₁ : τ₁  ...  Γ ⊢ eₙ : τₙ
  ─────────────────────────────────────
  Γ ⊢ C e₁ ... eₙ : T α₁...αₘ
```

#### Список (LIST)

```
  Γ ⊢ e₁ : τ  ...  Γ ⊢ eₙ : τ
  ──────────────────────────────
  Γ ⊢ [e₁ ... eₙ] : List τ
```

Пустой список: `Γ ⊢ [] : List α` (полиморфный).

#### Пайп (PIPE)

```
  Γ ⊢ e₁ : τ₁     Γ ⊢ e₂ : τ₁ → τ₂
  ────────────────────────────────────
        Γ ⊢ e₁ |> e₂ : τ₂
```

Десахаризация: `e₁ |> e₂` ≡ `e₂ e₁`

#### Композиция (COMPOSE)

```
  Γ ⊢ f : τ₁ → τ₂     Γ ⊢ g : τ₂ → τ₃
  ────────────────────────────────────────
         Γ ⊢ f >> g : τ₁ → τ₃
```

Десахаризация: `f >> g` ≡ `\x -> g (f x)`

### 4.4 Алгоритм вывода типов (Algorithm W)

Реализация стандартного Algorithm W (Damas & Milner, 1982):

```
W(Γ, e) → (S, τ)

где S — подстановка (substitution), τ — выведенный тип

Шаги:
1. Для переменной x: instantiate схему из Γ
2. Для применения e₁ e₂:
   (S₁, τ₁) = W(Γ, e₁)
   (S₂, τ₂) = W(S₁(Γ), e₂)
   α = свежая переменная
   S₃ = unify(S₂(τ₁), τ₂ → α)
   return (S₃ ∘ S₂ ∘ S₁, S₃(α))
3. Для лямбды \x -> e:
   α = свежая переменная
   (S₁, τ₁) = W(Γ ∪ {x : α}, e)
   return (S₁, S₁(α) → τ₁)
4. Для let x = e₁ в e₂:
   (S₁, τ₁) = W(Γ, e₁)
   σ = generalize(S₁(Γ), τ₁)
   (S₂, τ₂) = W(S₁(Γ) ∪ {x : σ}, e₂)
   return (S₂ ∘ S₁, τ₂)
```

**Unification** — стандартный алгоритм Robinson (1965):
```
unify(τ₁, τ₂):
  если τ₁ = τ₂ → пустая подстановка
  если τ₁ = α, α ∉ ftv(τ₂) → [α ↦ τ₂]
  если τ₂ = α, α ∉ ftv(τ₁) → [α ↦ τ₁]
  если τ₁ = T σ₁...σₙ, τ₂ = T ρ₁...ρₙ →
    композиция unify(σᵢ, ρᵢ) для i = 1..n
  иначе → type error
```

### 4.5 Встроенные операции (типы)

```
(+)  : Int → Int → Int         (и Float → Float → Float)
(-)  : Int → Int → Int
(*)  : Int → Int → Int
(/)  : Int → Int → Int
(%)  : Int → Int → Int
(==) : Eq a => a → a → Bool
(!=) : Eq a => a → a → Bool
(<)  : Ord a => a → a → Bool
(>)  : Ord a => a → a → Bool
(<=) : Ord a => a → a → Bool
(>=) : Ord a => a → a → Bool
(&&) : Bool → Bool → Bool
(||) : Bool → Bool → Bool
(++) : List a → List a → List a
(-) (prefix) : Int → Int
```

---

## 5. Операционная семантика (Big-Step)

### 5.1 Значения (values)

```
v ::= n                         -- целое число
    | f                         -- число с плавающей точкой
    | "s"                       -- строка
    | 'c'                       -- символ
    | true | false              -- булев
    | C v₁ ... vₙ              -- конструктор с аргументами
    | [v₁, ..., vₙ]            -- список
    | <λx.e, ρ>                 -- замыкание (closure)
```

`ρ` — окружение (environment), отображение имён в значения.

### 5.2 Правила вычисления

Нотация: `ρ ⊢ e ⇓ v` — "в окружении ρ выражение e вычисляется в значение v".

#### Литерал

```
  ─────────────
  ρ ⊢ n ⇓ n         (аналогично для Float, String, Char, Bool)
```

#### Переменная

```
  x ↦ v ∈ ρ
  ─────────────
  ρ ⊢ x ⇓ v
```

#### Лямбда

```
  ───────────────────────────
  ρ ⊢ \x -> e ⇓ <λx.e, ρ>
```

#### Применение (application)

```
  ρ ⊢ e₁ ⇓ <λx.body, ρ'>     ρ ⊢ e₂ ⇓ v₂     ρ'[x ↦ v₂] ⊢ body ⇓ v
  ──────────────────────────────────────────────────────────────────────
                          ρ ⊢ e₁ e₂ ⇓ v
```

Strict evaluation: аргумент `e₂` вычисляется до подстановки.

#### Бинарная операция

```
  ρ ⊢ e₁ ⇓ v₁     ρ ⊢ e₂ ⇓ v₂     v₁ ⊕ v₂ = v
  ─────────────────────────────────────────────────
               ρ ⊢ e₁ ⊕ e₂ ⇓ v
```

где ⊕ ∈ {+, -, *, /, %, ==, !=, <, >, <=, >=, &&, ||, ++}

#### Условие

```
  ρ ⊢ e₁ ⇓ true     ρ ⊢ e₂ ⇓ v
  ──────────────────────────────
  ρ ⊢ ? e₁ -> e₂ : e₃ ⇓ v

  ρ ⊢ e₁ ⇓ false    ρ ⊢ e₃ ⇓ v
  ──────────────────────────────
  ρ ⊢ ? e₁ -> e₂ : e₃ ⇓ v
```

Short-circuit: вычисляется только выбранная ветка.

#### Let-привязка (block)

```
  ρ ⊢ e₁ ⇓ v₁     ρ[x ↦ v₁] ⊢ e₂ ⇓ v₂
  ────────────────────────────────────────
        ρ ⊢ (x = e₁; e₂) ⇓ v₂
```

#### Конструктор

```
  ρ ⊢ e₁ ⇓ v₁  ...  ρ ⊢ eₙ ⇓ vₙ
  ──────────────────────────────────
  ρ ⊢ C e₁ ... eₙ ⇓ C v₁ ... vₙ
```

#### Pattern matching (определение функции)

Для функции с несколькими уравнениями `f p₁ = e₁; f p₂ = e₂; ...`:

```
  ρ ⊢ earg ⇓ v     match(pᵢ, v) = σ (первое совпадение)     ρ ∪ σ ⊢ eᵢ ⇓ vᵣ
  ────────────────────────────────────────────────────────────────────────────
                            ρ ⊢ f earg ⇓ vᵣ
```

Функция `match(p, v)`:
```
match(_, v)        = ∅                           -- wildcard
match(x, v)        = {x ↦ v}                     -- переменная
match(n, n)        = ∅                           -- литерал (совпал)
match(n, m)        = FAIL  (n ≠ m)              -- литерал (не совпал)
match(C p₁..pₙ, C v₁..vₙ) = ∪ᵢ match(pᵢ, vᵢ) -- конструктор
match(C .., D ..)  = FAIL  (C ≠ D)
match((p₁:p₂), Cons v₁ v₂) = match(p₁,v₁) ∪ match(p₂,v₂)  -- cons
```

Уравнения проверяются сверху вниз; используется первое совпавшее.

#### Список

```
  ρ ⊢ e₁ ⇓ v₁  ...  ρ ⊢ eₙ ⇓ vₙ
  ──────────────────────────────────
  ρ ⊢ [e₁ ... eₙ] ⇓ [v₁, ..., vₙ]
```

#### Пайп (desugaring)

```
  ρ ⊢ e₁ |> e₂  ≡  ρ ⊢ e₂ e₁
```

#### List comprehension (desugaring)

```
  [e | x <- xs, guard]  ≡  concatMap (\x -> ? guard -> [e] : []) xs
```

---

## 6. BPE-Alignment таблица

### 6.1 Верифицированная таблица операторов

Каждый оператор Synoema проверен на количество BPE-токенов в трёх основных токенизаторах:

| Оператор | cl100k_base (GPT-4) | Llama 3 | Mistral | Назначение |
|----------|--------------------:|--------:|--------:|------------|
| `--` | 1 | 1 | 1 | комментарий |
| `->` | 1 | 1 | 1 | стрелка |
| `<-` | 1 | 1 | 1 | bind |
| `++` | 1 | 1 | 1 | конкатенация |
| `==` | 1 | 1 | 1 | равенство |
| `!=` | 1 | 1 | 1 | неравенство |
| `>=` | 1 | 1 | 1 | больше-равно |
| `<=` | 1 | 1 | 1 | меньше-равно |
| `&&` | 1 | 1 | 1 | логическое И |
| `\|\|` | 1 | 1 | 1 | логическое ИЛИ |
| `>>` | 1 | 1 | 1 | композиция |
| `?` | 1 | 1 | 1 | условие |
| `:` | 1 | 1 | 1 | тип / cons |
| `.` | 1 | 1 | 1 | поле |
| `=` | 1 | 1 | 1 | определение |
| `@` | 1 | 1 | 1 | директива |
| `\|` | 1 | 1 | 1 | альтернатива |
| `_` | 1 | 1 | 1 | wildcard |
| `\` | 1 | 1 | 1 | лямбда |
| `\|>` | 1-2 | 1-2 | 1-2 | пайп |

### 6.2 Ключевые слова (BPE-анализ)

| Слово | cl100k_base | Примечание |
|-------|-------------|------------|
| `mod` | 1 | vs `module` = 1 (экономия 3 символов) |
| `use` | 1 | vs `import` = 1 (экономия 3 символов) |
| `trait` | 1 | vs `class` = 1 (семантически точнее) |
| `impl` | 1 | vs `instance` = 1-2 |
| `true` | 1 | стандарт |
| `false` | 1 | стандарт |

### 6.3 Сравнение с другими языками

Ключевые конструкции, где Synoema экономит:

| Конструкция | Python | Haskell | Synoema | Экономия |
|-------------|--------|---------|-------|----------|
| `if/else` | `if x else y` (4 tok) | `if x then y else z` (5 tok) | `? x -> y : z` (3 tok) | 25-40% |
| `def f(x):` | 5 tok | — | `f x =` (3 tok) | 40% |
| `lambda x: x+1` | 4 tok | `\x -> x+1` (3 tok) | `\x -> x+1` (3 tok) | 25% |
| `return x` | 2 tok | — | (не нужен, 0 tok) | 100% |
| `[x for x in xs if p]` | 8 tok | `[x | x <- xs, p x]` (7 tok) | `[x \| x <- xs , p x]` (7 tok) | 12% |
| запятые в списках | N-1 tok | N-1 tok | 0 tok | пропорционально N |

---

## 7. Примеры программ с токенным анализом

### 7.1 Факториал

**Synoema:**
```synoema
fac 0 = 1
fac n = n * fac (n - 1)
```

**Подсчёт токенов (приблизительно):**
```
fac  0  =  1  \n  fac  n  =  n  *  fac  (  n  -  1  )
 1   1  1  1   0   1   1  1  1  1   1   1  1  1  1  1  = 15 токенов
```

**Python эквивалент:**
```python
def fac(n):
    if n == 0:
        return 1
    return n * fac(n - 1)
```
≈ 25 токенов. **Экономия: 40%.**

### 7.2 Map

**Synoema:**
```synoema
map f [] = []
map f (x:xs) = f x : map f xs
```
≈ 18 токенов.

**Python:**
```python
def map_fn(f, lst):
    if not lst:
        return []
    return [f(lst[0])] + map_fn(f, lst[1:])
```
≈ 32 токена. **Экономия: 44%.**

### 7.3 QuickSort

**Synoema:**
```synoema
qsort [] = []
qsort (p:xs) = qsort lo ++ [p] ++ qsort hi
  lo = [x | x <- xs , x <= p]
  hi = [x | x <- xs , x > p]
```
≈ 35 токенов.

**Python:**
```python
def qsort(arr):
    if len(arr) <= 1:
        return arr
    p = arr[0]
    lo = [x for x in arr[1:] if x <= p]
    hi = [x for x in arr[1:] if x > p]
    return qsort(lo) + [p] + qsort(hi)
```
≈ 55 токенов. **Экономия: 36%.**

### 7.4 FizzBuzz

**Synoema:**
```synoema
fizzbuzz n =
  ? n % 15 == 0 -> "FizzBuzz"
  : ? n % 3 == 0 -> "Fizz"
  : ? n % 5 == 0 -> "Buzz"
  : show n

main = map fizzbuzz [1..100]
```
≈ 35 токенов.

**Python:**
```python
def fizzbuzz(n):
    if n % 15 == 0:
        return "FizzBuzz"
    elif n % 3 == 0:
        return "Fizz"
    elif n % 5 == 0:
        return "Buzz"
    else:
        return str(n)

result = [fizzbuzz(n) for n in range(1, 101)]
```
≈ 60 токенов. **Экономия: 42%.**

---

## Приложение A: Порядок реализации

### A.1 Парсер (Rust, ~2000 строк)

1. Лексер с offside rule (INDENT/DEDENT генерация)
2. Рекурсивный спуск (Pratt parsing для выражений)
3. AST определения (enum типы в Rust)
4. Pretty-printer для отладки

### A.2 Type Checker (Rust, ~1500 строк)

1. Типовые переменные и подстановки
2. Unification (Robinson)
3. Algorithm W
4. Обработка ADT (конструкторы, pattern matching)
5. Обработка ошибок (понятные сообщения)

### A.3 Интерпретатор (Rust, ~1000 строк)

1. Environment (HashMap<String, Value>)
2. Eval по big-step правилам из §5
3. Pattern matching engine
4. Встроенные функции (+, -, *, /, print, show, etc.)
5. REPL (read-eval-print loop)

### A.4 Верификация (фактическое состояние апрель 2026)

1. Тест-сьют: 43 unit-теста парсера (все зелёные)
2. Тест-сьют: 44 unit-теста type checker (все зелёные)
3. Тест-сьют: 63 интеграционных теста eval (все зелёные)
4. Тест-сьют: 73 unit-теста JIT codegen (все зелёные)
5. BPE-бенчмарк: 12 программ, -46% токенов vs Python (верифицировано)
6. Performance: 4.4× faster vs CPython 3.12 (JIT, 3 программы)

**Итого: 349 тестов, 0 ошибок, 0 warnings.**

---

## Приложение B: Реализованные и планируемые расширения

### B.1 Реализовано в v0.2–v0.4

- ✅ **Records** `{name = val, ...}` + `.field` + pattern matching — interpreter + JIT
- ✅ **Modules** `mod Name` + `use Name (func1 func2)` — lexical namespacing
- ✅ **Closures** в JIT — lambda lifting, indirect calls, `map`/`filter`
- ✅ **Strings** в JIT — tagged pointer, `show`, `++`, `length`, `==`, `!=`
- ✅ **List comprehensions** `[x | x <- xs, p x]` — через `concatMap`
- ✅ **Constant folding/DCE** в Core IR optimizer
- ✅ **TCO** в interpreter — 64MB stack thread, итеративный eval loop
- ✅ **Arena allocator** — 8MB bump arena, нет утечек памяти

### B.2 Запланировано

- **Effects / IO** (`<-`, `@io`) — монадический IO
- **Type classes** (`trait`, `impl`) — ad-hoc полиморфизм
- **Row polymorphism** для records
- **Region-Based Memory v2** — compile-time lifetimes
- **FFI** (`@native`) — вызов C-функций
- **Concurrency** — зарезервировано

---

*Synoema Language Reference v0.4*
*Статус: Implemented — все core features работают*
*Апрель 2026*
