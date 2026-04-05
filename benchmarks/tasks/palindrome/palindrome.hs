-- SPDX-License-Identifier: MIT-0
module Main where

isPalindrome :: Eq a => [a] -> Bool
isPalindrome xs = xs == reverse xs

showBool :: Bool -> String
showBool True  = "true"
showBool False = "false"

main :: IO ()
main = do
  putStrLn (showBool (isPalindrome [1 :: Int, 2, 3, 2, 1]))
  putStrLn (showBool (isPalindrome [1 :: Int, 2, 3]))
