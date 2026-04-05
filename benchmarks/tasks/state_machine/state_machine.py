# SPDX-License-Identifier: MIT-0

def next_state(s):
    return {"Green": "Yellow", "Yellow": "Red", "Red": "Green"}[s]

s = "Green"
for _ in range(3):
    n = next_state(s)
    print(f"{s} -> {n}")
    s = n
