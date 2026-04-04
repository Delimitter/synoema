# Spec: Map Type

## Текущее состояние

Нет Map в Synoema. Нужен для JSON, config, key-value storage.

## Design Decisions (решения по 6 проблемам)

### D1. Tuple problem → Pair ADT

Tuples `(a, b)` не существуют в парсере. Вместо добавления нового синтаксиса — определяем Pair в prelude:

```sno
Pair a b = MkPair a b

fst (MkPair a _) = a
snd (MkPair _ b) = b
```

Verified: nested parametric ADTs работают в type checker (`register_adt` + `resolve_type_expr`).

Map использует `[Pair k v]` внутри. Пользователь работает с `[MkPair k v]` для `from_list`.

### D2. Constructor hiding → не скрываем

Модули не поддерживают скрытие конструкторов. `MkMap` остаётся доступным.

**Mitigation:** документируем что `MkMap` — internal, не guaranteed API. Конвенция, не enforcement.

### D3. Name conflicts → prefix `map_`

`empty`, `insert`, `delete` — слишком generic. Используем prefix:

| Было (конфликтное) | Стало |
|--------------------|-------|
| `empty` | `map_empty` |
| `singleton` | `map_singleton` |
| `insert` | `map_insert` |
| `delete` | `map_delete` |
| `lookup` | `map_lookup` |
| `get` | `map_get` |
| `keys` | `map_keys` |
| `values` | `map_values` |
| `size` | `map_size` |

Исключения без prefix (уникальные): `from_pairs`, `has_key`, `entries`.

### D4. Sort → `list_sort` в prelude

Map — sorted assoc list, нужна сортировка. Добавляем `list_sort` в prelude:

```sno
--- Insertion sort. O(n²), sufficient for Map sizes (tens-hundreds).
list_sort [] = []
list_sort (x:xs) = list_insert_sorted x (list_sort xs)

list_insert_sorted x [] = [x]
list_insert_sorted x (y:ys) =
  ? x <= y -> x : y : ys
  : y : list_insert_sorted x ys
```

Verified: `<=` работает для Int и String (lexicographic) через `PartialOrd` в value.rs.

### D5. Ord constraint → runtime enforcement

Type checker типизирует `<` как `∀a. a → a → Bool` — без constraint. При runtime, comparison fails для non-comparable types (ADTs, records, functions).

**Решение:** Map работает с Int и String ключами (покрывает 95% use cases). Документируем ограничение. Typeclass `Ord` constraint — future work.

### D6. Full implementation code → provided below

## Type Definition

```sno
Pair a b = MkPair a b

fst (MkPair a _) = a
snd (MkPair _ b) = b

Map k v = MkMap [Pair k v]
```

## Full Implementation (prelude)

```sno
--- Empty map.
map_empty = MkMap []

--- Map with one entry.
map_singleton k v = MkMap [MkPair k v]

--- Build map from list of pairs. Deduplicates, keeps last.
from_pairs ps = foldl (\m p -> map_insert (fst p) (snd p) m) map_empty ps

--- Lookup key. Returns Result.
map_lookup k (MkMap []) = Err "key not found"
map_lookup k (MkMap ((MkPair k2 v):rest)) =
  ? k == k2 -> Ok v
  : ? k < k2 -> Err "key not found"
  : map_lookup k (MkMap rest)

--- Get with default.
map_get k def m = unwrap_or def (map_lookup k m)

--- Check if key exists.
has_key k m = is_ok (map_lookup k m)

--- Insert key-value (maintains sorted order).
map_insert k v (MkMap []) = MkMap [MkPair k v]
map_insert k v (MkMap ((MkPair k2 v2):rest)) =
  ? k == k2 -> MkMap (MkPair k v : rest)
  : ? k < k2 -> MkMap (MkPair k v : MkPair k2 v2 : rest)
  : map_insert_rest k v (MkPair k2 v2) rest

--- Helper: insert into rest, prepend kept element.
map_insert_rest k v kept [] = MkMap [kept, MkPair k v]
map_insert_rest k v kept ((MkPair k2 v2):rest) =
  ? k == k2 -> MkMap (kept : MkPair k v : rest)
  : ? k < k2 -> MkMap (kept : MkPair k v : MkPair k2 v2 : rest)
  : map_insert_rest_acc k v [kept, MkPair k2 v2] rest

--- Accumulator version to avoid deep recursion in MkMap wrapping.
--- NOTE: simplified — real implementation may need adjustment during apply.
map_insert_rest_acc k v acc [] = MkMap (reverse acc ++ [MkPair k v])
map_insert_rest_acc k v acc ((MkPair k2 v2):rest) =
  ? k == k2 -> MkMap (reverse acc ++ (MkPair k v : rest))
  : ? k < k2 -> MkMap (reverse acc ++ (MkPair k v : MkPair k2 v2 : rest))
  : map_insert_rest_acc k v (acc ++ [MkPair k2 v2]) rest

--- Delete key.
map_delete k (MkMap []) = MkMap []
map_delete k (MkMap ((MkPair k2 v2):rest)) =
  ? k == k2 -> MkMap rest
  : ? k < k2 -> MkMap (MkPair k2 v2 : rest)
  : MkMap (MkPair k2 v2 : map_delete_inner k rest)

map_delete_inner k [] = []
map_delete_inner k ((MkPair k2 v2):rest) =
  ? k == k2 -> rest
  : MkPair k2 v2 : map_delete_inner k rest

--- Update value at key (no-op if absent).
map_update k f (MkMap []) = MkMap []
map_update k f (MkMap ((MkPair k2 v2):rest)) =
  ? k == k2 -> MkMap (MkPair k (f v2) : rest)
  : MkMap (MkPair k2 v2 : map_update_inner k f rest)

map_update_inner k f [] = []
map_update_inner k f ((MkPair k2 v2):rest) =
  ? k == k2 -> MkPair k2 (f v2) : rest
  : MkPair k2 v2 : map_update_inner k f rest

--- Traversal.
map_keys (MkMap ps) = map (\p -> fst p) ps
map_values (MkMap ps) = map (\p -> snd p) ps
entries (MkMap ps) = ps
map_size (MkMap ps) = length ps

--- Transform values.
map_map_values f (MkMap ps) = MkMap (map (\p -> MkPair (fst p) (f (snd p))) ps)

--- Fold over entries.
map_fold f acc (MkMap []) = acc
map_fold f acc (MkMap ((MkPair k v):rest)) = map_fold f (f acc k v) (MkMap rest)

--- Merge two maps (right map wins on conflict).
map_merge (MkMap []) m2 = m2
map_merge m1 (MkMap []) = m1
map_merge m1 (MkMap ((MkPair k v):rest)) = map_merge (map_insert k v m1) (MkMap rest)
```

## Зависимости

- `Result` (prelude) — для `map_lookup`
- `foldl` (builtin) — для `from_pairs`
- `map`, `length`, `reverse` (builtins) — для traversal
- `fst`, `snd` (prelude, Pair) — для accessors
- `==`, `<` — для key comparison (Int, String only at runtime)

## API Summary

| Function | Type | Notes |
|----------|------|-------|
| `map_empty` | `Map k v` | |
| `map_singleton` | `k -> v -> Map k v` | |
| `from_pairs` | `[Pair k v] -> Map k v` | |
| `map_lookup` | `k -> Map k v -> Result v String` | O(n) |
| `map_get` | `k -> v -> Map k v -> v` | with default |
| `has_key` | `k -> Map k v -> Bool` | |
| `map_insert` | `k -> v -> Map k v -> Map k v` | O(n) |
| `map_delete` | `k -> Map k v -> Map k v` | O(n) |
| `map_update` | `k -> (v -> v) -> Map k v -> Map k v` | no-op if absent |
| `map_keys` | `Map k v -> [k]` | sorted |
| `map_values` | `Map k v -> [v]` | |
| `entries` | `Map k v -> [Pair k v]` | |
| `map_size` | `Map k v -> Int` | |
| `map_map_values` | `(v -> w) -> Map k v -> Map k w` | |
| `map_fold` | `(b -> k -> v -> b) -> b -> Map k v -> b` | |
| `map_merge` | `Map k v -> Map k v -> Map k v` | right wins |

## Complexity

Sorted association list: O(n) insert/lookup/delete. Sufficient for tens-hundreds of entries.

## Ограничения (документировать)

1. Ключи должны поддерживать `==` и `<` (Int, String). Другие типы → runtime error
2. `MkMap` конструктор доступен пользователю (convention: не использовать напрямую)
3. O(n) complexity — не для больших наборов данных

## Что НЕ входит

- Hash-based Map (нарушает минимализм)
- Set (эмулируется: `Map k ()`)
- Typeclass `Ord` constraint (future)
- Tuple syntax `(a, b)` — используем Pair ADT
