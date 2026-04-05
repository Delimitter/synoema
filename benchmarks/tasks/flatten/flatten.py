# SPDX-License-Identifier: MIT-0

def flatten(nested):
    if isinstance(nested, list):
        result = []
        for item in nested:
            result.extend(flatten(item))
        return result
    return [nested]

nested = [1, [2, 3], [4, [5, 6]]]
result = flatten(nested)
print("[" + " ".join(str(x) for x in result) + "]")
