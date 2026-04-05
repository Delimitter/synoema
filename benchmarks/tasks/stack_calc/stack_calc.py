# SPDX-License-Identifier: MIT-0

def eval_rpn(ops):
    stack = []
    for op in ops:
        if isinstance(op, tuple) and op[0] == "push":
            stack.append(op[1])
        elif op == "add":
            b, a = stack.pop(), stack.pop()
            stack.append(a + b)
        elif op == "mul":
            b, a = stack.pop(), stack.pop()
            stack.append(a * b)
    return stack[0]

result = eval_rpn([("push", 3), ("push", 4), "add", ("push", 5), "mul"])
print(result)
