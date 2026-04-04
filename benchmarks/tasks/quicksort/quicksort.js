// SPDX-License-Identifier: MIT-0

function qsort(lst) {
    if (lst.length === 0) return [];
    const [p, ...xs] = lst;
    const lo = xs.filter(x => x <= p);
    const hi = xs.filter(x => x > p);
    return [...qsort(lo), p, ...qsort(hi)];
}

const result = qsort([5, 3, 8, 1, 9, 2, 7, 4, 6]);
console.log("[" + result.join(" ") + "]");
