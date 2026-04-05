-- SPDX-License-Identifier: MIT-0

divide :: Int -> Int -> Either String Int
divide _ 0 = Left "division by zero"
divide x y = Right (x `div` y)

showResult :: Either String Int -> String
showResult (Left e)  = "Err " ++ e
showResult (Right v) = "Ok " ++ show v

main :: IO ()
main = do
  putStrLn (showResult (divide 10 0))
  putStrLn (showResult (divide 10 2))
