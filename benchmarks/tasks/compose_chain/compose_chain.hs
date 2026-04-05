-- SPDX-License-Identifier: MIT-0
module Main where

double :: Int -> Int
double x = x * 2

inc :: Int -> Int
inc x = x + 1

square :: Int -> Int
square x = x * x

transform :: Int -> Int
transform = square . inc . double

showList' :: Show a => [a] -> String
showList' xs = "[" ++ unwords (map show xs) ++ "]"

main :: IO ()
main = putStrLn (showList' (map transform [1..5]))
