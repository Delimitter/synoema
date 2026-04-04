// SPDX-License-Identifier: MIT-0

function fib(n: number): number {
    if (n === 0) return 0;
    if (n === 1) return 1;
    return fib(n - 1) + fib(n - 2);
}

console.log(fib(30));
