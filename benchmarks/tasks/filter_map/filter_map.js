// SPDX-License-Identifier: MIT-0

const result = Array.from({length: 10}, (_, i) => i + 1)
    .filter(x => x % 2 === 0)
    .map(x => x * x)
    .reduce((a, b) => a + b, 0);

console.log(result);
