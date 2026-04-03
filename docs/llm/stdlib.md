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
```

## String
```
str_len         : String -> Int
str_slice       : String -> Int -> Int -> String    -- str from to
str_find        : String -> String -> Int -> Int    -- str sub start (-1 = not found)
str_starts_with : String -> String -> Bool
str_trim        : String -> String
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

## Concurrency (interpreter only)
```
chan  : Chan a                 -- create typed channel
send : Chan a -> a -> ()
recv : Chan a -> a
```
