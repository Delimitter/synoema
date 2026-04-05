-- SPDX-License-Identifier: MIT-0

main :: IO ()
main = print (sum (map (\x -> x * x) (filter even [1..10])))
