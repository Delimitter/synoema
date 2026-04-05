-- SPDX-License-Identifier: MIT-0

fac :: Int -> Int
fac 0 = 1
fac n = n * fac (n - 1)

main :: IO ()
main = print (fac 10)
