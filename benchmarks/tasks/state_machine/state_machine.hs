-- SPDX-License-Identifier: MIT-0
module Main where

data Light = Green | Yellow | Red

showLight :: Light -> String
showLight Green  = "Green"
showLight Yellow = "Yellow"
showLight Red    = "Red"

next :: Light -> Light
next Green  = Yellow
next Yellow = Red
next Red    = Green

transition :: Light -> String
transition l = showLight l ++ " -> " ++ showLight (next l)

main :: IO ()
main = do
  putStrLn (transition Green)
  putStrLn (transition Yellow)
  putStrLn (transition Red)
