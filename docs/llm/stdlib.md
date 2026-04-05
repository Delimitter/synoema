# Stdlib

## List
```
length    : [a] -> Int
head      : [a] -> a              -- error on []
tail      : [a] -> [a]            -- error on []
sum       : [Int] -> Int
map       : (a -> b) -> [a] -> [b]
filter    : (a -> Bool) -> [a] -> [a]
foldl     : (b -> a -> b) -> b -> [a] -> b
concatMap : (a -> [b]) -> [a] -> [b]
zip       : [a] -> [b] -> [(a, b)]  -- pair elements, stop at shorter
index     : Int -> [a] -> a          -- 0-based, error on out-of-bounds
take      : Int -> [a] -> [a]        -- first n elements
drop      : Int -> [a] -> [a]        -- skip first n elements
reverse   : [a] -> [a]              -- reverse list
for_each  : (a -> b) -> [a] -> ()   -- apply f to each element (side effects)
```

## String
```
str_len         : String -> Int
str_slice       : String -> Int -> Int -> String    -- str from to
str_find        : String -> String -> Int -> Int    -- str sub start (-1 = not found)
str_starts_with : String -> String -> Bool
str_trim        : String -> String
str_join        : String -> [String] -> String  -- join with separator: str_join ", " ["a" "b"] = "a, b"
json_escape     : String -> String
```

## Math
```
sqrt  : Float -> Float         -- also Int -> Float
floor : Float -> Float
ceil  : Float -> Float
round : Float -> Float
abs   : Int -> Int             -- also Float -> Float
even  : Int -> Bool
odd   : Int -> Bool
```

## Logic
```
not : Bool -> Bool
```

## Result (prelude — always available)
```
Result a e = Ok a | Err e

map_ok     : (a -> b) -> Result a e -> Result b e
map_err    : (e -> f) -> Result a e -> Result a f
unwrap     : Result a e -> a            -- error on Err
unwrap_or  : a -> Result a e -> a
is_ok      : Result a e -> Bool
is_err     : Result a e -> Bool
and_then   : (a -> Result b e) -> Result a e -> Result b e
error      : String -> a                -- runtime panic
```

## IO
```
print    : a -> ()             -- print + newline
show     : a -> String         -- any type to string
readline : String              -- read line from stdin
```

## File / Network (interpreter only)
```
file_read      : String -> String   -- read entire file
fd_open        : String -> Fd       -- open file for reading (streaming)
fd_open_write  : String -> Fd       -- open file for writing
tcp_listen     : Int -> Fd
tcp_accept     : Fd -> Fd
fd_readline    : Fd -> String
fd_write       : Fd -> String -> ()
fd_close       : Fd -> ()
fd_popen       : String -> Fd
```

## Environment
```
env    : String -> String              -- env var (empty if missing)
env_or : String -> String -> String    -- env var with default
args   : [String]                      -- CLI args after --
```

## Map (prelude — sorted assoc list)
```
Pair a b = MkPair a b
Map k v  = MkMap (List (Pair k v))

map_empty      : Map k v
map_singleton  : k -> v -> Map k v
map_insert     : k -> v -> Map k v -> Map k v
map_lookup     : k -> Map k v -> Result v String
map_get        : k -> v -> Map k v -> v         -- with default
has_key        : k -> Map k v -> Bool
map_delete     : k -> Map k v -> Map k v
map_update     : k -> (v -> v) -> Map k v -> Map k v
from_pairs     : [Pair k v] -> Map k v
map_keys       : Map k v -> [k]                 -- sorted
map_values     : Map k v -> [v]
entries        : Map k v -> [Pair k v]
map_size       : Map k v -> Int
map_map_values : (v -> w) -> Map k v -> Map k w
map_merge      : Map k v -> Map k v -> Map k v
fst            : Pair a b -> a
snd            : Pair a b -> b
```

## JSON
```
JsonValue = JNull | JBool Bool | JNum Int | JStr String
          | JArr [JsonValue] | JObj [Pair String JsonValue]

json_parse  : String -> Result JsonValue String
json_encode : JsonValue -> String              -- JIT only
json_get    : String -> JsonValue -> Result JsonValue String  -- single flat key, NOT dot-path
json_str    : JsonValue -> String              -- extract String, error on mismatch
json_num    : JsonValue -> Int                 -- extract number, error on mismatch
json_arr    : JsonValue -> [JsonValue]         -- extract array, error on mismatch
json_obj    : JsonValue -> [Pair String JsonValue]  -- extract object pairs
json_escape : String -> String
```

Notes:
- `JNum` stores `Int` for integers, `Float` for decimals (e.g. `3.14`)
- `json_get` takes ONE key, not dot-paths. Nest calls for deep access:
  `json_get "b" (unwrap (json_get "a" obj))`
- `JObj` pairs are sorted by key (compatible with `map_lookup_list`)

## Concurrency (interpreter only)
```
chan  : Chan a                 -- create typed channel
send : Chan a -> a -> ()
recv : Chan a -> a
```
