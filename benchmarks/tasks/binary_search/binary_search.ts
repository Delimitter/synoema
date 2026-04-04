// SPDX-License-Identifier: MIT-0

function binarySearch(xs: number[], target: number): number {
    let lo = 0, hi = xs.length - 1;
    while (lo <= hi) {
        const mid = Math.floor((lo + hi) / 2);
        if (xs[mid] === target) return mid;
        else if (xs[mid] < target) lo = mid + 1;
        else hi = mid - 1;
    }
    return -1;
}

console.log(binarySearch([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 7));
