# SPDX-License-Identifier: MIT-0

def scan_left(f, acc, xs):
    result = [acc]
    for x in xs:
        acc = f(acc, x)
        result.append(acc)
    return result

result = scan_left(lambda a, b: a + b, 0, [1, 2, 3, 4, 5])
print("[" + " ".join(str(x) for x in result) + "]")
