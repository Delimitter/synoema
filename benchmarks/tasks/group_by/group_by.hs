-- SPDX-License-Identifier: MIT-0
module Main where

showList' :: Show a => [a] -> String
showList' xs = "[" ++ unwords (map show xs) ++ "]"

partition' :: (a -> Bool) -> [a] -> ([a], [a])
partition' _ [] = ([], [])
partition' p (x:xs)
  | p x       = (x : yes, no)
  | otherwise  = (yes, x : no)
  where (yes, no) = partition' p xs

main :: IO ()
main = do
  let (evens, odds) = partition' (\x -> x `mod` 2 == 0) [1..10 :: Int]
  putStrLn (showList' evens)
  putStrLn (showList' odds)
