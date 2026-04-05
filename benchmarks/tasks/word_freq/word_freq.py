# SPDX-License-Identifier: MIT-0

words = "the cat sat on the mat the cat".split()

def count(w, ws):
    return sum(1 for x in ws if x == w)

print(f"the: {count('the', words)}")
print(f"cat: {count('cat', words)}")
print(f"sat: {count('sat', words)}")
