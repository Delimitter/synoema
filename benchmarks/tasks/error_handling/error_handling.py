# SPDX-License-Identifier: MIT-0

def divide(x, y):
    if y == 0:
        return ("Err", "division by zero")
    return ("Ok", x // y)

def show_result(r):
    if r[0] == "Err":
        return f"Err {r[1]}"
    return f"Ok {r[1]}"

print(show_result(divide(10, 0)))
print(show_result(divide(10, 2)))
