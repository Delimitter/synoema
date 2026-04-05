-- SPDX-License-Identifier: MIT-0

binarySearch :: [Int] -> Int -> Int
binarySearch xs target = go 0 (length xs - 1)
  where
    go lo hi
      | lo > hi        = -1
      | val == target  = mid
      | val < target   = go (mid + 1) hi
      | otherwise      = go lo (mid - 1)
      where
        mid = (lo + hi) `div` 2
        val = xs !! mid

main :: IO ()
main = print (binarySearch [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] 7)
