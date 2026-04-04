// SPDX-License-Identifier: MIT-0

function collatz(n) {
    if (n === 1) return 0;
    if (n % 2 === 0) return 1 + collatz(n / 2);
    return 1 + collatz(3 * n + 1);
}

console.log(collatz(27));
