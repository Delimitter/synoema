# SPDX-License-Identifier: MIT-0

def merge(xs, ys):
    if len(xs) == 0:
        return ys
    if len(ys) == 0:
        return xs
    if xs[0] <= ys[0]:
        return [xs[0]] + merge(xs[1:], ys)
    else:
        return [ys[0]] + merge(xs, ys[1:])

def msort(lst):
    if len(lst) <= 1:
        return lst
    n = len(lst) // 2
    left = lst[:n]
    right = lst[n:]
    return merge(msort(left), msort(right))

result = msort([5, 3, 8, 1, 9, 2, 7, 4, 6])
print("[" + " ".join(str(x) for x in result) + "]")
