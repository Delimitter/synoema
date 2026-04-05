-- SPDX-License-Identifier: MIT-0
module Main where

data Result a b = Err a | Ok b

validateName :: String -> Result String String
validateName "" = Err "empty name"
validateName n  = Ok n

validateAge :: Int -> Result String Int
validateAge a
  | a > 0 && a < 150 = Ok a
  | otherwise         = Err "invalid age"

showResult :: Result String String -> String
showResult (Ok s)  = "Ok " ++ s
showResult (Err s) = "Err " ++ s

main :: IO ()
main = do
  putStrLn (showResult (validate "Alice" 25))
  putStrLn (showResult (validate "" 25))
  where
    validate name age =
      case validateName name of
        Err e -> Err e
        Ok n  -> case validateAge age of
          Err e -> Err e
          Ok a  -> Ok (n ++ " is " ++ show a)
