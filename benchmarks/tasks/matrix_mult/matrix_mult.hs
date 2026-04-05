-- SPDX-License-Identifier: MIT-0

matMul :: [[Int]] -> [[Int]] -> [[Int]]
matMul a b =
  let n = length a
      col j = map (!! j) b
      dot r c = sum (zipWith (*) r c)
  in [[dot (a !! i) (col j) | j <- [0..n-1]] | i <- [0..n-1]]

main :: IO ()
main =
  let a = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
      b = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
      result = matMul a b
      flat = concatMap id result
  in putStrLn (unwords (map show flat))
