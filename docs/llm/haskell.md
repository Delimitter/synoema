# Haskell -- Quick Reference for LLM Code Generation

> Optimized reference for generating correct Haskell (GHC 9.x) code in algorithmic tasks.

---

## 1. Overrides -- read these FIRST

| Instead of ... | Write ... | Why |
|---|---|---|
| deep parentheses `f (g (h x))` | `f $ g $ h x` or `f . g . h $ x` | `$` = low-precedence apply |
| `let` in function body | `where` clause after `=` | idiomatic Haskell style |
| `if c then x else y` (nested) | guards `\| c = x` | cleaner multi-branch |
| manual recursion on lists | `map`/`filter`/`fold` | more readable + fusible |
| `String` in performance code | `Data.Text` + `OverloadedStrings` | `String = [Char]` is slow |
| `head xs` / `tail xs` | pattern match `(x:xs)` | `head []` crashes at runtime |
| `read s :: Int` | `readMaybe s :: Maybe Int` | `read` crashes on bad input |
| omitting type signature | always add top-level `::` | documents intent, catches bugs |
| tabs | spaces (2 or 4) | tabs cause parse errors in Haskell |
| `return x` at end of `do` | `pure x` | works in any `Applicative`, not just `Monad` |

---

## 2. Axioms

1. Purely functional -- no mutation; all "changes" produce new values
2. Lazy evaluation -- values computed only when needed; enables infinite structures
3. Type inference (Hindley-Milner) -- compiler infers types; annotations optional but recommended
4. Pattern matching everywhere -- function equations, `case`, `let`, `where`
5. Currying by default -- `f :: a -> b -> c` takes args one at a time; partial application is free
6. `main :: IO ()` -- entry point; all side effects live in `IO` monad
7. Indentation-sensitive -- `where`, `let`, `do`, `case` blocks use layout rule

---

## 3. Functions & Pattern Matching

```haskell
-- type signature + equations
factorial :: Integer -> Integer
factorial 0 = 1
factorial n = n * factorial (n - 1)

-- guards
bmi :: Double -> String
bmi x
  | x < 18.5  = "underweight"
  | x < 25.0  = "normal"
  | otherwise  = "overweight"

-- where clause
circleArea :: Double -> Double
circleArea r = pi * r ^ 2
  where pi = 3.14159265

-- let..in (expression-level)
cylVol r h = let base = pi * r ^ 2 in base * h

-- lambda
square :: Int -> Int
square = \x -> x * x

-- pattern matching in case
descr :: [a] -> String
descr xs = case xs of
  []    -> "empty"
  [_]   -> "singleton"
  _     -> "multiple"

-- as-pattern: xs@(x:_) binds both whole and parts
firstAndAll xs@(x:_) = (x, xs)
```

---

## 4. Core Types & Data

```haskell
-- ADT
data Shape = Circle Double | Rect Double Double | Point
  deriving (Show, Eq)

-- record syntax (auto-generates accessors)
data Person = Person { name :: String, age :: Int }
  deriving (Show)

-- newtype (zero-cost wrapper, one constructor, one field)
newtype Meters = Meters Double deriving (Show, Eq, Ord)

-- type alias (transparent)
type Name = String
type Pair a b = (a, b)

-- Maybe / Either
safeDivide :: Int -> Int -> Maybe Int
safeDivide _ 0 = Nothing
safeDivide x y = Just (x `div` y)

parseAge :: String -> Either String Int
parseAge s = case readMaybe s of
  Nothing -> Left $ "bad age: " ++ s
  Just n  -> Right n

-- pattern matching on Maybe/Either
fromMaybe :: a -> Maybe a -> a
fromMaybe def Nothing  = def
fromMaybe _   (Just x) = x
```

---

## 5. Lists

```haskell
xs    = [1, 2, 3, 4, 5]
empty = [] :: [Int]
range = [1..10]              -- [1,2,...,10]
step  = [1,3..10]            -- [1,3,5,7,9]
inf   = [1..]                -- infinite (lazy)
cat   = [1,2] ++ [3,4]      -- concatenation
cons  = 0 : [1, 2]          -- [0,1,2]

-- comprehension
evens = [x | x <- [1..20], even x]
pairs = [(x,y) | x <- [1..3], y <- [1..3], x /= y]

-- pattern matching
myHead :: [a] -> Maybe a
myHead []    = Nothing
myHead (x:_) = Just x

-- common operations (see section 7)
result = map (*2) . filter even $ [1..10]   -- [4,8,12,16,20]
total  = foldl' (+) 0 [1..100]             -- 5050 (strict fold)
```

---

## 6. IO & do-notation

```haskell
main :: IO ()
main = do
  putStrLn "Enter name:"
  name <- getLine
  let greeting = "Hello, " ++ name ++ "!"
  putStrLn greeting

-- sequence actions; use mapM_ for side effects
printAll :: [String] -> IO ()
printAll = mapM_ putStrLn
```

---

## 7. Stdlib Essentials

| Function | Type | Notes |
|---|---|---|
| `show` | `Show a => a -> String` | convert to string |
| `putStrLn` / `print` | `String -> IO ()` / `Show a => a -> IO ()` | output |
| `getLine` | `IO String` | read line |
| `map` | `(a -> b) -> [a] -> [b]` | transform |
| `filter` | `(a -> Bool) -> [a] -> [a]` | select |
| `foldl'` | `(b -> a -> b) -> b -> [a] -> b` | strict left fold (use over `foldl`) |
| `foldr` | `(a -> b -> b) -> b -> [a] -> b` | right fold (works on infinite lists) |
| `concatMap` | `(a -> [b]) -> [a] -> [b]` | map + flatten |
| `length` | `[a] -> Int` | O(n) |
| `head` / `tail` | partial -- crash on `[]` | prefer pattern match |
| `take` / `drop` | `Int -> [a] -> [a]` | first/skip n |
| `zip` | `[a] -> [b] -> [(a,b)]` | stops at shorter |
| `zipWith` | `(a -> b -> c) -> [a] -> [b] -> [c]` | zip + apply |
| `reverse` | `[a] -> [a]` | reverse list |
| `sort` | `Ord a => [a] -> [a]` | `Data.List.sort` |
| `nub` | `Eq a => [a] -> [a]` | remove duplicates (O(n^2)) |
| `words` / `unwords` | `String -> [String]` / `[String] -> String` | split/join on spaces |
| `lines` / `unlines` | split/join on newlines | similar to words |
| `fromMaybe` | `a -> Maybe a -> a` | unwrap with default |
| `either` | `(a -> c) -> (b -> c) -> Either a b -> c` | case on Either |
| `Data.Map.fromList/lookup/insert` | `Ord k => ...` | `import qualified Data.Map as M` |
| `Data.Set.fromList` | `Ord a => [a] -> Set a` | `import qualified Data.Set as S` |

---

## 8. Gotchas

1. **`foldl` is lazy** -- use `Data.List.foldl'` (strict) to avoid stack overflow on large lists
2. **`head`/`tail` are partial** -- crash on `[]`; pattern match or use `Data.Maybe.listToMaybe`
3. **`String` = `[Char]`** -- very slow; use `Data.Text` for real programs
4. **Monomorphism restriction** -- add type signatures to top-level bindings to avoid surprises
5. **`$` precedence** -- `f $ g $ h x` = `f (g (h x))`; `$` is right-assoc, lowest precedence
6. **Lazy space leaks** -- `let acc = acc + x` builds thunks; use `seq`, `BangPatterns`, or strict fields
7. **Tabs cause errors** -- Haskell layout rule requires spaces
8. **`/` is fractional** -- integer division is `` `div` `` or `` `mod` ``; `/` requires `Fractional`
9. **`Data.List.sort` needs import** -- not in `Prelude`; `import Data.List (sort, nub, group)`
10. **`do` alignment** -- all statements in a `do` block must start at the same column
