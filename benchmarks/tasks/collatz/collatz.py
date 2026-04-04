# SPDX-License-Identifier: MIT-0

def collatz(n):
    if n == 1:
        return 0
    if n % 2 == 0:
        return 1 + collatz(n // 2)
    return 1 + collatz(3 * n + 1)

print(collatz(27))
