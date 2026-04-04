# SPDX-License-Identifier: MIT-0

def qsort(lst):
    if len(lst) == 0:
        return []
    p = lst[0]
    xs = lst[1:]
    lo = [x for x in xs if x <= p]
    hi = [x for x in xs if x > p]
    return qsort(lo) + [p] + qsort(hi)

result = qsort([5, 3, 8, 1, 9, 2, 7, 4, 6])
print("[" + " ".join(str(x) for x in result) + "]")
