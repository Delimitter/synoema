-- SPDX-License-Identifier: MIT-0
module Main where

splitOn :: Char -> String -> [String]
splitOn _ [] = [""]
splitOn d (c:cs)
  | c == d    = "" : splitOn d cs
  | otherwise = let (w:ws) = splitOn d cs in (c:w) : ws

main :: IO ()
main = putStrLn result
  where
    fields = splitOn ',' "Alice,30,NYC"
    result = case fields of
      [n, a, c] -> n ++ " is " ++ a ++ " from " ++ c
      _         -> "parse error"
