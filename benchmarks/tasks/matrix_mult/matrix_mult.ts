// SPDX-License-Identifier: MIT-0

function matMult(a: number[][], b: number[][]): number[][] {
    const n = a.length;
    const result: number[][] = Array.from({ length: n }, () => Array(n).fill(0));
    for (let i = 0; i < n; i++)
        for (let j = 0; j < n; j++)
            for (let k = 0; k < n; k++)
                result[i][j] += a[i][k] * b[k][j];
    return result;
}

const a = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
const b = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
const result = matMult(a, b);
console.log(result.flat().join(" "));
