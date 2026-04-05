# Python -- Quick Reference for LLM Code Generation

> Optimized reference for generating correct Python 3.10+ code in algorithmic tasks.

---

## 1. Overrides -- read these FIRST

| Instead of ... | Write ... | Why |
|---|---|---|
| bare script code | `if __name__ == "__main__": main()` | importable + runnable |
| `print(result)` in functions | `return result` | callers need values, not side effects |
| `except:` (bare) | `except Exception as e:` | bare catches KeyboardInterrupt/SystemExit |
| `dict["key"]` | `dict.get("key", default)` | avoids KeyError |
| `list == []` | `not list` | pythonic emptiness check |
| `type(x) == int` | `isinstance(x, int)` | handles subclasses |
| manual loops for transform | list comprehension | faster + idiomatic |
| nested `if/elif` chains | `match/case` (3.10+) | cleaner pattern matching |
| `lambda x: x["k"]` | `operator.itemgetter("k")` | clearer intent in sort/map |
| mutable default `def f(xs=[])` | `def f(xs=None)` then `xs = xs or []` | mutable defaults shared across calls |

---

## 2. Axioms

1. Python 3.10+ -- use `match/case`, `|` union types, `(x := val)` walrus
2. Indentation = blocks (4 spaces, never tabs)
3. Everything is an object; duck typing; EAFP over LBYL
4. Comprehensions are expressions: `[expr for x in it if cond]`
5. `None` is falsy; `0`, `""`, `[]`, `{}` are falsy
6. Assignment is statement, not expression (except `:=`)
7. Slicing: `s[start:stop:step]` -- stop is exclusive, negative indices from end

---

## 3. Functions & Pattern Matching

```python
# definition + type hints
def gcd(a: int, b: int) -> int:
    while b:
        a, b = b, a % b
    return a

# lambda (single expression only)
square = lambda x: x * x

# default + keyword args
def connect(host: str, port: int = 8080, *, timeout: float = 30.0) -> None: ...

# *args, **kwargs
def wrap(*args, **kwargs): return func(*args, **kwargs)

# match/case (3.10+)
def classify(val):
    match val:
        case 0:                     return "zero"
        case int(n) if n > 0:      return "positive"
        case [x, *rest]:            return f"list starting with {x}"
        case {"type": "circle", "r": r}: return f"circle r={r}"
        case _:                     return "other"
```

---

## 4. Control Flow

```python
# conditional expression
x = "even" if n % 2 == 0 else "odd"

# for + else (else runs if no break)
for item in items:
    if pred(item): break
else:
    handle_not_found()

# walrus operator (3.8+)
if (n := len(data)) > 10:
    process(data, n)

# comprehensions
squares = [x**2 for x in range(10)]
evens   = [x for x in nums if x % 2 == 0]
flat    = [y for xs in nested for y in xs]
lookup  = {k: v for k, v in pairs if v is not None}
uniq    = {x for x in items}
```

---

## 5. Data Types

```python
# dataclass (preferred for structured data)
from dataclasses import dataclass, field

@dataclass
class Point:
    x: float
    y: float

    def dist(self) -> float:
        return (self.x**2 + self.y**2) ** 0.5

# frozen (immutable, hashable)
@dataclass(frozen=True)
class Color:
    r: int; g: int; b: int

# NamedTuple alternative
from typing import NamedTuple
class Pair(NamedTuple):
    fst: int
    snd: int

# type hints (3.10+)
def process(items: list[int | str]) -> dict[str, list[int]]: ...
```

---

## 6. Error Handling

```python
# try/except/else/finally
try:
    result = compute(data)
except ValueError as e:
    result = fallback
except (TypeError, KeyError):
    raise
else:
    save(result)       # runs only if no exception
finally:
    cleanup()          # always runs

# raise with context
raise ValueError(f"invalid: {x!r}") from original_error

# custom exception
class AppError(Exception):
    def __init__(self, msg: str, code: int = 0):
        super().__init__(msg)
        self.code = code
```

---

## 7. Stdlib Essentials

| Function | Usage | Notes |
|---|---|---|
| `len(x)` | length of str/list/dict/set | O(1) |
| `range(start, stop, step)` | half-open `[start, stop)` | stop exclusive |
| `enumerate(it, start=0)` | `(index, val)` pairs | use in for loops |
| `zip(a, b, strict=True)` | pair elements | `strict` errors on unequal len |
| `sorted(it, key=, reverse=)` | new sorted list | stable sort |
| `map(f, it)` / `filter(f, it)` | lazy iterators | wrap in `list()` if needed |
| `sum(it)` / `min(it)` / `max(it)` | aggregators | `min/max` take `key=` |
| `any(it)` / `all(it)` | short-circuit bool | take generator exprs |
| `isinstance(x, (int, str))` | type check | tuple = OR |
| `str.split(sep)` | split string | default = whitespace |
| `sep.join(it)` | join strings | `", ".join(words)` |
| `str.strip()` | trim whitespace | `.lstrip()`, `.rstrip()` |
| `str.find(sub)` | index or `-1` | `.index()` raises |
| `str.replace(old, new)` | replace all | returns new string |
| `list.append(x)` | add to end | mutates |
| `list.extend(it)` | add all from iterable | mutates |
| `list.pop(i=-1)` | remove + return | mutates |
| `dict.get(k, default)` | safe lookup | `None` if no default |
| `dict.items()` / `.keys()` / `.values()` | views | iterable |
| `collections.Counter(it)` | frequency map | `.most_common(n)` |
| `collections.defaultdict(type)` | auto-init dict | `defaultdict(list)` |
| `itertools.chain(*its)` | flatten iterables | lazy |
| `functools.lru_cache` | memoize | `@lru_cache(maxsize=None)` |
| `heapq.heappush/heappop` | min-heap | `heapq.nlargest(n, it)` |
| `bisect.insort/bisect_left` | sorted list ops | O(log n) search |

---

## 8. Gotchas

1. **Mutable default args** -- `def f(xs=[])` shares one list across all calls; use `None`
2. **Integer division** -- `7 / 2 = 3.5` (float); `7 // 2 = 3` (floor); `-7 // 2 = -4`
3. **Strings are immutable** -- `.replace()` returns new string, does not mutate
4. **0-indexed** -- `xs[0]` is first; `xs[-1]` is last; `xs[1:3]` = indices 1,2
5. **`is` vs `==`** -- `is` checks identity; `==` checks equality; use `is None`, `== 0`
6. **Shallow copy** -- `xs[:]`, `list(xs)`, `.copy()` are shallow; use `copy.deepcopy` for nested
7. **Late binding closures** -- `[lambda: i for i in range(3)]` all return 2; fix: `lambda i=i: i`
8. **`dict` preserves insertion order** (3.7+) but `set` does not
9. **`in` on list is O(n)** -- use `set` for membership; `in` on `dict` checks keys
10. **Unpacking gotcha** -- single-element tuple: `(x,)` not `(x)`
