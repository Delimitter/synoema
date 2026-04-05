-- SPDX-License-Identifier: MIT-0
module Main where

data BST = Empty | Node Int BST BST

insert :: Int -> BST -> BST
insert x Empty = Node x Empty Empty
insert x (Node v left right)
  | x < v    = Node v (insert x left) right
  | x > v    = Node v left (insert x right)
  | otherwise = Node v left right

inorder :: BST -> [Int]
inorder Empty            = []
inorder (Node v left right) = inorder left ++ [v] ++ inorder right

main :: IO ()
main = putStrLn (unwords (map show (inorder tree)))
  where
    tree = foldr insert Empty [4, 1, 7, 3, 5]
