# SPDX-License-Identifier: MIT-0

class Leaf:
    def __init__(self, value):
        self.value = value

class Node:
    def __init__(self, left, value, right):
        self.left = left
        self.value = value
        self.right = right

def inorder(tree):
    if isinstance(tree, Leaf):
        return [tree.value]
    return inorder(tree.left) + [tree.value] + inorder(tree.right)

tree = Node(Node(Leaf(1), 3, Leaf(4)), 5, Node(Leaf(6), 7, Leaf(9)))
print(" ".join(str(x) for x in inorder(tree)))
