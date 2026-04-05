-- SPDX-License-Identifier: MIT-0
module Main where

data Op = Push Int | Add | Mul

eval :: [Op] -> [Int] -> Int
eval [] (x:_)             = x
eval (Push n : ops) stack = eval ops (n : stack)
eval (Add : ops) (a:b:s)  = eval ops ((b + a) : s)
eval (Mul : ops) (a:b:s)  = eval ops ((b * a) : s)
eval _ _                  = error "invalid"

main :: IO ()
main = print (eval [Push 3, Push 4, Add, Push 5, Mul] [])
