-- SPDX-License-Identifier: MIT-0
module Main where

data Nested a = Flat a | Nested [Nested a]

flatten :: Nested a -> [a]
flatten (Flat x)    = [x]
flatten (Nested xs) = concatMap flatten xs

showList' :: Show a => [a] -> String
showList' xs = "[" ++ unwords (map show xs) ++ "]"

main :: IO ()
main = putStrLn (showList' (flatten structure))
  where
    structure = Nested [ Nested [Flat (1 :: Int), Flat 2, Flat 3]
                       , Nested [Flat 4, Flat 5]
                       , Flat 6
                       ]
