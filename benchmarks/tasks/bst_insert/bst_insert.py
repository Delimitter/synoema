# SPDX-License-Identifier: MIT-0

def insert(tree, v):
    if tree is None:
        return (None, v, None)
    left, val, right = tree
    if v < val:
        return (insert(left, v), val, right)
    elif v > val:
        return (left, val, insert(right, v))
    return tree

def inorder(tree):
    if tree is None:
        return []
    left, val, right = tree
    return inorder(left) + [val] + inorder(right)

tree = None
for v in [5, 3, 7, 1, 4]:
    tree = insert(tree, v)

print(" ".join(str(x) for x in inorder(tree)))
