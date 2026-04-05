-- SPDX-License-Identifier: MIT-0
module Main where

showList' :: Show a => [a] -> String
showList' xs = "[" ++ unwords (map show xs) ++ "]"

main :: IO ()
main = putStrLn (showList' (scanl (+) 0 [1..5 :: Int]))
