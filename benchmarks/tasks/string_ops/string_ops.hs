-- SPDX-License-Identifier: MIT-0

main :: IO ()
main =
  let s = "Hello" ++ " World"
      l = length s
  in putStrLn (s ++ " " ++ show l)
