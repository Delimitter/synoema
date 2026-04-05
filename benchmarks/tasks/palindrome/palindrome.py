# SPDX-License-Identifier: MIT-0

def palindrome(xs):
    return xs == xs[::-1]

print(str(palindrome([1, 2, 3, 2, 1])).lower())
print(str(palindrome([1, 2, 3])).lower())
