// SPDX-License-Identifier: MIT-0

type List<T> = { type: "nil" } | { type: "cons"; head: T; tail: List<T> };

function nil<T>(): List<T> { return { type: "nil" }; }
function cons<T>(head: T, tail: List<T>): List<T> { return { type: "cons", head, tail }; }

function length<T>(xs: List<T>): number {
    if (xs.type === "nil") return 0;
    return 1 + length(xs.tail);
}

function append<T>(xs: List<T>, x: T): List<T> {
    if (xs.type === "nil") return cons(x, nil());
    return cons(xs.head, append(xs.tail, x));
}

const xs = cons(1, cons(2, cons(3, nil<number>())));
const ys = append(xs, 4);
console.log(length(ys));
