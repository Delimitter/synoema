-- SPDX-License-Identifier: MIT-0

gcd' :: Int -> Int -> Int
gcd' a 0 = a
gcd' a b = gcd' b (a `mod` b)

main :: IO ()
main = print (gcd' 48 18)
