# SPDX-License-Identifier: MIT-0

def safe_div(x, y):
    if y == 0:
        return None
    return x // y

def map_maybe(f, m):
    if m is None:
        return None
    return f(m)

def show_maybe(m):
    if m is None:
        return "Nothing"
    return f"Just {m}"

r1 = map_maybe(lambda x: x * 3, safe_div(10, 2))
r2 = map_maybe(lambda x: x * 3, safe_div(10, 0))
print(show_maybe(r1))
print(show_maybe(r2))
