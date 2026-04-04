// SPDX-License-Identifier: MIT-0

function gcd(a: number, b: number): number {
    if (b === 0) return a;
    return gcd(b, a % b);
}

console.log(gcd(48, 18));
