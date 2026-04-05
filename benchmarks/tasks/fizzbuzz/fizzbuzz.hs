-- SPDX-License-Identifier: MIT-0

fizzbuzz :: Int -> String
fizzbuzz n
  | n `mod` 15 == 0 = "FizzBuzz"
  | n `mod` 3  == 0 = "Fizz"
  | n `mod` 5  == 0 = "Buzz"
  | otherwise        = show n

main :: IO ()
main = putStrLn (fizzbuzz 15)
