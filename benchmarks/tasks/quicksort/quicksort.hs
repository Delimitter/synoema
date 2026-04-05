-- SPDX-License-Identifier: MIT-0

qsort :: [Int] -> [Int]
qsort [] = []
qsort (p:xs) =
  let lo = [x | x <- xs, x <= p]
      hi = [x | x <- xs, x > p]
  in qsort lo ++ [p] ++ qsort hi

showList' :: [Int] -> String
showList' xs = "[" ++ unwords (map show xs) ++ "]"

main :: IO ()
main = putStrLn (showList' (qsort [5, 3, 8, 1, 9, 2, 7, 4, 6]))
