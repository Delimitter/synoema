-- SPDX-License-Identifier: MIT-0
module Main where

data Person = Person { name :: String, x :: Int, y :: Int }

display :: Person -> String
display p = name p ++ " at (" ++ show (x p) ++ ", " ++ show (y p) ++ ")"

main :: IO ()
main = putStrLn (display updated)
  where
    person  = Person { name = "Alice", x = 1, y = 2 }
    updated = person { name = "Bob", x = 3, y = 4 }
