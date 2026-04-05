-- SPDX-License-Identifier: MIT-0

collatz :: Int -> Int
collatz 1 = 0
collatz n
  | n `mod` 2 == 0 = 1 + collatz (n `div` 2)
  | otherwise       = 1 + collatz (3 * n + 1)

main :: IO ()
main = print (collatz 27)
