-- SPDX-License-Identifier: MIT-0
module Main where

count :: String -> [String] -> Int
count w = length . filter (== w)

main :: IO ()
main = do
  let ws = words "the cat sat on the mat the cat"
  putStrLn ("the: " ++ show (count "the" ws))
  putStrLn ("cat: " ++ show (count "cat" ws))
  putStrLn ("sat: " ++ show (count "sat" ws))
