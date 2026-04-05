-- SPDX-License-Identifier: MIT-0

merge :: [Int] -> [Int] -> [Int]
merge [] ys = ys
merge xs [] = xs
merge (x:xs) (y:ys)
  | x <= y   = x : merge xs (y:ys)
  | otherwise = y : merge (x:xs) ys

msort :: [Int] -> [Int]
msort [] = []
msort [x] = [x]
msort xs =
  let n = length xs `div` 2
      (left, right) = splitAt n xs
  in merge (msort left) (msort right)

showList' :: [Int] -> String
showList' xs = "[" ++ unwords (map show xs) ++ "]"

main :: IO ()
main = putStrLn (showList' (msort [5, 3, 8, 1, 9, 2, 7, 4, 6]))
