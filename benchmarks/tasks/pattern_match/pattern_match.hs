-- SPDX-License-Identifier: MIT-0

data Shape = Circle Double | Rectangle Double Double | Triangle Double Double

area :: Shape -> Double
area (Circle r)      = 3.14 * r * r
area (Rectangle w h) = w * h
area (Triangle b h)  = 0.5 * b * h

showArea :: Double -> String
showArea x
  | x == fromIntegral (round x) = show (round x :: Int)
  | otherwise                    = show x

describe :: String -> Shape -> String
describe name shape = name ++ ": " ++ showArea (area shape)

main :: IO ()
main = do
  putStrLn (describe "Circle" (Circle 5.0))
  putStrLn (describe "Rectangle" (Rectangle 4.0 6.0))
  putStrLn (describe "Triangle" (Triangle 6.0 5.0))
