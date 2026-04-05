# SPDX-License-Identifier: MIT-0

def double(x): return x * 2
def inc(x): return x + 1
def square(x): return x * x

def compose(*fns):
    def composed(x):
        for f in fns:
            x = f(x)
        return x
    return composed

pipeline = compose(double, inc, square)
result = [pipeline(x) for x in [1, 2, 3, 4, 5]]
print("[" + " ".join(str(x) for x in result) + "]")
