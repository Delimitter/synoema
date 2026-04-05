-- SPDX-License-Identifier: MIT-0
module Main where

power :: Int -> Int -> Int
power _ 0 = 1
power b n = b * power b (n - 1)

main :: IO ()
main = print (power 2 10)
