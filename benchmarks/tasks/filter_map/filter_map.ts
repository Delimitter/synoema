// SPDX-License-Identifier: MIT-0

const result: number = Array.from({length: 10}, (_, i) => i + 1)
    .filter((x: number) => x % 2 === 0)
    .map((x: number) => x * x)
    .reduce((a: number, b: number) => a + b, 0);

console.log(result);
