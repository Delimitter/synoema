-- SPDX-License-Identifier: MIT-0
module Main where

safeDiv :: Int -> Int -> Maybe Int
safeDiv _ 0 = Nothing
safeDiv a b = Just (a `div` b)

showResult :: Maybe Int -> String
showResult Nothing  = "Nothing"
showResult (Just x) = "Just " ++ show x

main :: IO ()
main = do
  putStrLn (showResult (fmap (* 3) (safeDiv 10 2)))
  putStrLn (showResult (fmap (* 3) (safeDiv 10 0)))
