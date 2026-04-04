// SPDX-License-Identifier: MIT-0

function merge(xs, ys) {
    if (xs.length === 0) return ys;
    if (ys.length === 0) return xs;
    if (xs[0] <= ys[0]) return [xs[0], ...merge(xs.slice(1), ys)];
    return [ys[0], ...merge(xs, ys.slice(1))];
}

function msort(lst) {
    if (lst.length <= 1) return lst;
    const n = Math.floor(lst.length / 2);
    const left = lst.slice(0, n);
    const right = lst.slice(n);
    return merge(msort(left), msort(right));
}

const result = msort([5, 3, 8, 1, 9, 2, 7, 4, 6]);
console.log("[" + result.join(" ") + "]");
