# SPDX-License-Identifier: MIT-0

class Nil:
    pass

class Cons:
    def __init__(self, head, tail):
        self.head = head
        self.tail = tail

def length(xs):
    if isinstance(xs, Nil):
        return 0
    return 1 + length(xs.tail)

def append(xs, x):
    if isinstance(xs, Nil):
        return Cons(x, Nil())
    return Cons(xs.head, append(xs.tail, x))

xs = Cons(1, Cons(2, Cons(3, Nil())))
ys = append(xs, 4)
print(length(ys))
