-- SPDX-License-Identifier: MIT-0

data List a = Nil | Cons a (List a)

len :: List a -> Int
len Nil        = 0
len (Cons _ t) = 1 + len t

append :: List a -> a -> List a
append Nil x        = Cons x Nil
append (Cons h t) x = Cons h (append t x)

main :: IO ()
main =
  let xs = Cons 1 (Cons 2 (Cons 3 Nil))
      ys = append xs 4
  in print (len ys)
