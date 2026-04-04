// SPDX-License-Identifier: MIT-0

function divide(x, y) {
    if (y === 0) return { tag: "Err", value: "division by zero" };
    return { tag: "Ok", value: Math.floor(x / y) };
}

function showResult(r) {
    return r.tag === "Err" ? `Err ${r.value}` : `Ok ${r.value}`;
}

console.log(showResult(divide(10, 0)));
console.log(showResult(divide(10, 2)));
