-- SPDX-License-Identifier: MIT-0

data Tree a = Leaf a | Node (Tree a) a (Tree a)

inorder :: Tree a -> [a]
inorder (Leaf x)       = [x]
inorder (Node l v r)   = inorder l ++ [v] ++ inorder r

main :: IO ()
main =
  let tree = Node (Node (Leaf 1) 3 (Leaf 4)) 5 (Node (Leaf 6) 7 (Leaf 9))
  in putStrLn (unwords (map show (inorder tree)))
