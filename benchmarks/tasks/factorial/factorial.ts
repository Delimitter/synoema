// SPDX-License-Identifier: MIT-0

function fac(n: number): number {
    if (n === 0) return 1;
    return n * fac(n - 1);
}

console.log(fac(10));
