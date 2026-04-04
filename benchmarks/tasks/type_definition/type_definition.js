// SPDX-License-Identifier: MIT-0

function nil() { return { type: "nil" }; }
function cons(head, tail) { return { type: "cons", head, tail }; }

function length(xs) {
    if (xs.type === "nil") return 0;
    return 1 + length(xs.tail);
}

function append(xs, x) {
    if (xs.type === "nil") return cons(x, nil());
    return cons(xs.head, append(xs.tail, x));
}

const xs = cons(1, cons(2, cons(3, nil())));
const ys = append(xs, 4);
console.log(length(ys));
