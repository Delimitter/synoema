// SPDX-License-Identifier: MIT-0

type Result<T, E> = { tag: "Ok"; value: T } | { tag: "Err"; value: E };

function divide(x: number, y: number): Result<number, string> {
    if (y === 0) return { tag: "Err", value: "division by zero" };
    return { tag: "Ok", value: Math.floor(x / y) };
}

function showResult(r: Result<number, string>): string {
    return r.tag === "Err" ? `Err ${r.value}` : `Ok ${r.value}`;
}

console.log(showResult(divide(10, 0)));
console.log(showResult(divide(10, 2)));
