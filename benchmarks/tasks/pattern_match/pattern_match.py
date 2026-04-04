# SPDX-License-Identifier: MIT-0

def area(shape):
    if shape[0] == "Circle":
        r = shape[1]
        return 3.14 * r * r
    elif shape[0] == "Rectangle":
        w, h = shape[1], shape[2]
        return w * h
    elif shape[0] == "Triangle":
        b, h = shape[1], shape[2]
        return 0.5 * b * h

shapes = [
    ("Circle", "Circle", 5.0),
    ("Rectangle", "Rectangle", 4.0, 6.0),
    ("Triangle", "Triangle", 6.0, 5.0),
]

for name, *args in [("Circle", 5.0), ("Rectangle", 4.0, 6.0), ("Triangle", 6.0, 5.0)]:
    shape = (name, *args)
    a = area(shape)
    result = int(a) if a == int(a) else a
    print(f"{name}: {result}")
