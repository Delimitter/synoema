// SPDX-License-Identifier: MIT-0

function area(shape) {
    switch (shape.type) {
        case "Circle": return 3.14 * shape.r * shape.r;
        case "Rectangle": return shape.w * shape.h;
        case "Triangle": return 0.5 * shape.b * shape.h;
    }
}

function fmt(n) { return n === Math.floor(n) ? String(Math.floor(n)) : String(n); }

const shapes = [
    { type: "Circle", name: "Circle", r: 5.0 },
    { type: "Rectangle", name: "Rectangle", w: 4.0, h: 6.0 },
    { type: "Triangle", name: "Triangle", b: 6.0, h: 5.0 },
];

for (const s of shapes) {
    console.log(`${s.name}: ${fmt(area(s))}`);
}
