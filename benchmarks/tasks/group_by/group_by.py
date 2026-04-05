# SPDX-License-Identifier: MIT-0

nums = list(range(1, 11))
evens = [x for x in nums if x % 2 == 0]
odds = [x for x in nums if x % 2 != 0]
print("[" + " ".join(str(x) for x in evens) + "]")
print("[" + " ".join(str(x) for x in odds) + "]")
